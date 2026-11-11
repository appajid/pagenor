//! Locating and editing items, threading, and story management.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use pagenor_geom::{Affine, Point, Rect, shapes};
use serde::{Deserialize, Serialize};

use crate::ids::{ItemId, LayerId, StoryId};
use crate::item::{Content, Item, Shape, TextFrame, TextFrameOptions};
use crate::page::Spread;
use crate::story::{CharRun, ParaFormat, Story};
use crate::{DocError, Document, Result};

/// Which spread list an item lives in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", tag = "kind", content = "index")]
pub enum SpreadRef {
    Doc(usize),
    Parent(usize),
}

/// Indices from `spread.items` down through nested groups.
pub type ItemPath = Vec<usize>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ItemLoc {
    pub spread: SpreadRef,
    pub path: ItemPath,
}

impl ItemLoc {
    /// Top-level index in the spread.
    pub fn top(&self) -> usize {
        self.path[0]
    }
}

/// Pre-order traversal with heap-held iterators, bounded by the resident tree's depth.
/// A group cannot recursively own itself through immutable `Arc<Item>` children.
/// The predicate stops the walk as soon as all requested locations are found.
fn locate_in(items: &[Arc<Item>], path: &mut ItemPath, mut found: impl FnMut(&Item, &ItemPath) -> bool) -> bool {
    let mut levels = vec![items.iter().enumerate()];
    while let Some(level) = levels.last_mut() {
        if let Some((i, item)) = level.next() {
            path.push(i);
            if found(item, path) {
                return true;
            }
            let children = item.children();
            if children.is_empty() {
                path.pop();
            } else {
                levels.push(children.iter().enumerate());
            }
        } else {
            levels.pop();
            path.pop();
        }
    }
    false
}

fn find_in(items: &[Arc<Item>], id: ItemId, path: &mut ItemPath) -> bool {
    locate_in(items, path, |item, _| item.id == id)
}

impl Document {
    pub fn spread(&self, r: SpreadRef) -> Option<&Spread> {
        match r {
            SpreadRef::Doc(i) => self.spreads.get(i).map(|s| &**s),
            SpreadRef::Parent(i) => self.parents.get(i).map(|s| &**s),
        }
    }

    pub fn spread_mut(&mut self, r: SpreadRef) -> Option<&mut Spread> {
        match r {
            SpreadRef::Doc(i) => self.spreads.get_mut(i).map(Arc::make_mut),
            SpreadRef::Parent(i) => self.parents.get_mut(i).map(Arc::make_mut),
        }
    }

    /// All spread refs: document spreads then parents.
    pub fn spread_refs(&self) -> impl Iterator<Item = SpreadRef> {
        (0..self.spreads.len()).map(SpreadRef::Doc).chain((0..self.parents.len()).map(SpreadRef::Parent))
    }

    pub fn find(&self, id: ItemId) -> Option<ItemLoc> {
        for r in self.spread_refs() {
            let mut path = Vec::new();
            if find_in(&self.spread(r)?.items, id, &mut path) {
                return Some(ItemLoc { spread: r, path });
            }
        }
        None
    }

    /// Locate a resident batch of IDs in one pre-order document walk, stopping once
    /// all are found. Missing IDs are absent; repeated IDs resolve once. As with
    /// `find`, document spreads precede parents and the first matching item wins.
    /// Work is O(items + IDs + returned path lengths), with no recursive traversal.
    /// The slice bounds request storage to IDs already held by the caller.
    pub fn find_many(&self, ids: &[ItemId]) -> HashMap<ItemId, ItemLoc> {
        let mut out = HashMap::new();
        if ids.is_empty() {
            return out;
        }
        // A single distinct ID needs only cheap comparisons, not a hash per item.
        let mut want: HashSet<ItemId> = ids.iter().copied().collect();
        if want.len() == 1 {
            if let Some(id) = want.iter().copied().next()
                && let Some(loc) = self.find(id)
            {
                out.insert(id, loc);
            }
            return out;
        }
        for spread in self.spread_refs() {
            if let Some(sp) = self.spread(spread) {
                let complete = locate_in(&sp.items, &mut Vec::new(), |item, path| {
                    if want.remove(&item.id) {
                        out.insert(item.id, ItemLoc { spread, path: path.clone() });
                    }
                    want.is_empty()
                });
                if complete {
                    break;
                }
            }
        }
        out
    }

    pub fn item_at(&self, loc: &ItemLoc) -> Option<&Item> {
        let sp = self.spread(loc.spread)?;
        let mut it: &Item = sp.items.get(*loc.path.first()?)?;
        for &i in &loc.path[1..] {
            it = it.children().get(i)?;
        }
        Some(it)
    }

    pub fn item(&self, id: ItemId) -> Option<&Item> {
        self.item_at(&self.find(id)?)
    }

    /// Repair stale thread references without discarding the story or its text.
    /// Build ownership once, including grouped/master/anchored items, rather than searching
    /// the document for every referenced frame. Other invalid document data still fails check.
    pub fn drop_dangling_story_frames(&mut self) {
        let mut owned = HashMap::new();
        let mut items: Vec<&Item> = self.spreads.iter().chain(&self.parents).flat_map(|s| s.items.iter().map(AsRef::as_ref)).rev().collect();
        while let Some(item) = items.pop() {
            owned.entry(item.id).or_insert_with(|| item.text_frame().map(|t| t.story));
            items.extend(item.children().iter().rev().map(AsRef::as_ref));
        }
        for (_, item) in self.anchored_items() {
            owned.entry(item.id).or_insert_with(|| item.text_frame().map(|t| t.story));
        }
        for (sid, story) in &mut self.stories {
            if story.frames.iter().any(|f| owned.get(f).copied().flatten() != Some(*sid)) {
                Arc::make_mut(story).frames.retain(|f| owned.get(f).copied().flatten() == Some(*sid));
            }
        }
    }

    /// Items embedded in stories (including footnotes and table cells), with their
    /// owning top-level story. Does not follow text-frame references between stories.
    pub fn anchored_items(&self) -> Vec<(StoryId, &Item)> {
        let mut out = Vec::new();
        let mut stories: Vec<_> = self.stories.iter().map(|(id, story)| (*id, story.as_ref())).collect();
        while let Some((owner, story)) = stories.pop() {
            stories.extend(story.notes.iter().chain(&story.endnotes).map(|n| (owner, &n.text)));
            stories.extend(story.tables.values().flat_map(|t| t.cells.iter().map(|c| (owner, &c.text))));
            let mut items: Vec<&Item> = story.objects.iter().map(|o| &o.item).collect();
            while let Some(item) = items.pop() {
                out.push((owner, item));
                items.extend(item.children().iter().map(AsRef::as_ref));
            }
        }
        out
    }

    /// An embedded item is not a spread selection location (`find`).
    pub fn anchored_item(&self, id: ItemId) -> Option<&Item> {
        self.anchored_items().into_iter().find_map(|(_, item)| (item.id == id).then_some(item))
    }

    pub fn item_mut_at(&mut self, loc: &ItemLoc) -> Option<&mut Item> {
        let sp = self.spread_mut(loc.spread)?;
        let mut it: &mut Item = Arc::make_mut(sp.items.get_mut(*loc.path.first()?)?);
        for &i in &loc.path[1..] {
            it = Arc::make_mut(it.children_mut()?.get_mut(i)?);
        }
        Some(it)
    }

    pub fn item_mut(&mut self, id: ItemId) -> Option<&mut Item> {
        let loc = self.find(id)?;
        self.item_mut_at(&loc)
    }

    /// The top-level item (group root) containing `id`.
    pub fn top_level_of(&self, id: ItemId) -> Option<ItemId> {
        let loc = self.find(id)?;
        Some(self.spread(loc.spread)?.items[loc.top()].id)
    }

    /// The immediate parent group of `id`, if `id` is nested inside a group. Only groups hold
    /// items on a spread (see [`Item::children`]); anchored items live in stories and have no
    /// parent here.
    pub fn parent_of(&self, id: ItemId) -> Option<ItemId> {
        let chain = self.ancestry_chain(id);
        let (last, parents) = chain.split_last()?;
        if *last != id {
            return None;
        }
        parents.last().copied()
    }

    /// The item a Selection-tool click on the leaf `leaf` picks: the deepest ancestor (or `leaf`
    /// itself) that `is_selected` says is already selected, so a click inside a selected group
    /// keeps drilling at that level, otherwise the top-level item. Clicks and the context menu
    /// share this so they agree.
    pub fn selection_target(&self, leaf: ItemId, is_selected: impl Fn(ItemId) -> bool) -> ItemId {
        self.ancestry_chain(leaf).into_iter().rev().find(|&i| is_selected(i)).or_else(|| self.top_level_of(leaf)).unwrap_or(leaf)
    }

    /// Item IDs along the ancestry path from top-level down to `id` (inclusive); empty when `id`
    /// is not on a spread.
    pub fn ancestry_chain(&self, id: ItemId) -> Vec<ItemId> {
        let Some(loc) = self.find(id) else { return vec![] };
        let Some(sp) = self.spread(loc.spread) else { return vec![] };
        let mut items: &[Arc<Item>] = &sp.items;
        let mut chain = Vec::with_capacity(loc.path.len());
        for &i in &loc.path {
            let Some(it) = items.get(i) else { break };
            chain.push(it.id);
            items = it.children();
        }
        chain
    }

    /// Spread-space transform of the parent group chain of the item at `loc` (identity at top level).
    pub fn parent_xf(&self, loc: &ItemLoc) -> pagenor_geom::Affine {
        let mut xf = pagenor_geom::Affine::IDENTITY;
        if let Some(sp) = self.spread(loc.spread) {
            let mut items = &sp.items;
            for &i in &loc.path[..loc.path.len().saturating_sub(1)] {
                let it = &items[i];
                xf *= it.child_space();
                items = match &it.content {
                    Content::Group { items } => items,
                    _ => break,
                };
            }
        }
        xf
    }

    /// Insert an item into a spread at `index` (None = front).
    pub fn insert_item(&mut self, r: SpreadRef, item: Item, index: Option<usize>) -> Result<ItemId> {
        let id = item.id;
        let sp = self.spread_mut(r).ok_or_else(|| DocError::Invalid(format!("no spread {r:?}")))?;
        let i = index.unwrap_or(sp.items.len()).min(sp.items.len());
        sp.items.insert(i, Arc::new(item));
        Ok(id)
    }

    /// Remove an item (and its descendants) from the document. Text frames leave their threads;
    /// stories that lose their last frame are deleted.
    pub fn remove_item(&mut self, id: ItemId) -> Result<Arc<Item>> {
        let loc = self.find(id).ok_or(DocError::NoItem(id))?;
        let removed = {
            let sp = self.spread_mut(loc.spread).ok_or(DocError::NoItem(id))?;
            if loc.path.len() == 1 {
                sp.items.remove(loc.path[0])
            } else {
                let mut it: &mut Item = Arc::make_mut(&mut sp.items[loc.path[0]]);
                for &i in &loc.path[1..loc.path.len() - 1] {
                    it = Arc::make_mut(&mut it.children_mut().ok_or(DocError::NoItem(id))?[i]);
                }
                let last = *loc.path.last().ok_or(DocError::NoItem(id))?;
                it.children_mut().ok_or(DocError::NoItem(id))?.remove(last)
            }
        };
        let mut frames = Vec::new();
        removed.walk(&mut |i| {
            if let Some(t) = i.text_frame() {
                frames.push((i.id, t.story));
            }
        });
        for (fid, sid) in frames {
            let empty = {
                let st = self.story_mut(sid).ok_or(DocError::NoStory(sid))?;
                st.frames.retain(|f| *f != fid);
                st.frames.is_empty()
            };
            if empty {
                self.stories.remove(&sid);
            }
        }
        Ok(removed)
    }

    /// The column direction a new story starts with: right to left in a document whose binding is
    /// right to left (Document Setup), else left to right. Stories already made keep theirs.
    pub fn new_story_direction(&self) -> crate::TextDirection {
        if self.settings.right_to_left_binding { crate::TextDirection::RightToLeft } else { crate::TextDirection::LeftToRight }
    }

    /// Create a text frame with a new (or given) story at `rect` on spread `r`.
    pub fn add_text_frame(&mut self, r: SpreadRef, rect: Rect, layer: LayerId, text: &str, para: ParaFormat) -> Result<(ItemId, StoryId)> {
        let id = ItemId(self.alloc());
        let sid = StoryId(self.alloc());
        let mut story = Story::with_text(sid, text, para);
        story.direction = self.new_story_direction();
        story.frames.push(id);
        self.stories.insert(sid, Arc::new(story));
        let mut item = Item::new(id, layer, Shape::Rectangle, shapes::rectangle(rect));
        item.object_style = self.styles.default_text_frame.clone();
        if let Some(os) = self.styles.object_style(&self.styles.default_text_frame).cloned() {
            if let Some(f) = os.fill {
                item.fill = f;
            }
            if let Some(s) = os.stroke {
                item.stroke = s;
            }
        }
        item.content = Content::Text(TextFrame { story: sid, options: TextFrameOptions::default() });
        self.insert_item(r, item, None)?;
        Ok((id, sid))
    }

    /// Thread `from` → `to`: `to` joins `from`'s story right after `from` (between `from` and the
    /// frame that followed it, if any). If `to` held its own text, that text is appended to the
    /// story (as new paragraphs) and its old story is removed; frames that followed `to` come
    /// along, frames before it keep an empty story of their own. An empty (unassigned) `from` or
    /// `to` becomes a text frame. `from`'s story keeps its id.
    pub fn thread(&mut self, from: ItemId, to: ItemId) -> Result<()> {
        if from == to {
            return Err(DocError::Invalid("cannot thread a frame to itself".into()));
        }
        self.check_thread_links(from, to, false)?;
        let ts = self.frame_story(to)?;
        let fs = match self.frame_story(from)? {
            Some(s) => s,
            None => self.give_empty_story(from)?,
        };
        if ts == Some(fs) {
            return Err(DocError::Invalid("frames are already in the same story".into()));
        }
        let Some(ts) = ts else {
            let st = self.story_mut(fs).ok_or(DocError::NoStory(fs))?;
            let at = st.frames.iter().position(|f| *f == from).map_or(st.frames.len(), |p| p + 1);
            st.frames.insert(at, to);
            st.rev = st.rev.wrapping_add(1);
            let it = self.item_mut(to).ok_or(DocError::NoItem(to))?;
            it.content = Content::Text(TextFrame { story: fs, options: TextFrameOptions::default() });
            return Ok(());
        };
        let mut other = Arc::unwrap_or_clone(self.stories.remove(&ts).ok_or(DocError::NoStory(ts))?);
        // Frames of the other story from `to` on follow along; frames before it are left as an empty new story.
        let pos_to = other.frames.iter().position(|f| *f == to).unwrap_or(0);
        let after = other.frames.split_off(pos_to);
        let before = std::mem::take(&mut other.frames);
        let (vertical, direction) = (other.vertical, other.direction);
        let offset;
        {
            let st = self.story_mut(fs).ok_or(DocError::NoStory(fs))?;
            offset = if st.text.is_empty() {
                0
            } else if other.text.is_empty() {
                st.len()
            } else {
                st.len().saturating_add(1)
            };
            // The story keeps its own direction, even when it was empty and takes the other's text.
            let own = (st.vertical, st.direction);
            st.append_story(other);
            (st.vertical, st.direction) = own;
            let at = st.frames.iter().position(|f| *f == from).map_or(st.frames.len(), |p| p + 1);
            for (k, f) in after.iter().enumerate() {
                st.frames.insert(at + k, *f);
            }
            st.rev = st.rev.wrapping_add(1);
        }
        self.remap_thread_story_refs(ts, fs, offset);
        for f in &after {
            if let Some(t) = self.item_mut(*f).and_then(Item::text_frame_mut) {
                t.story = fs;
            }
        }
        if !before.is_empty() {
            let nid = StoryId(self.alloc());
            let mut ns = Story::new(nid);
            ns.frames = before.clone();
            ns.vertical = vertical;
            ns.direction = direction;
            for f in &before {
                if let Some(t) = self.item_mut(*f).and_then(Item::text_frame_mut) {
                    t.story = nid;
                }
            }
            self.stories.insert(nid, Arc::new(ns));
        }
        Ok(())
    }

    /// [`Document::thread`], except that an empty `from` alone in its story, threaded in front of
    /// the first frame of `to`'s story, joins that story: the story with the text keeps its id
    /// (and whatever refers to it).
    pub fn thread_keeping_story(&mut self, from: ItemId, to: ItemId) -> Result<()> {
        let fs = self.frame_story(from)?;
        let from_empty = fs.is_none_or(|fs| self.story(fs).is_some_and(|s| s.text.is_empty() && s.frames == [from]));
        let ts = self.frame_story(to)?;
        let to_first = ts.and_then(|ts| self.story(ts)).is_some_and(|s| s.frames.first() == Some(&to));
        let (Some(ts), true, true) = (ts, from_empty, to_first && from != to) else { return self.thread(from, to) };
        self.check_thread_links(from, to, true)?;
        if let Some(fs) = fs {
            self.stories.remove(&fs);
            self.remap_thread_story_refs(fs, ts, 0);
        }
        let st = self.story_mut(ts).ok_or(DocError::NoStory(ts))?;
        st.frames.insert(0, from);
        st.rev = st.rev.wrapping_add(1);
        let it = self.item_mut(from).ok_or(DocError::NoItem(from))?;
        match it.text_frame_mut() {
            Some(t) => t.story = ts,
            None => it.content = Content::Text(TextFrame { story: ts, options: TextFrameOptions::default() }),
        }
        Ok(())
    }

    fn remap_thread_story_refs(&mut self, from: StoryId, to: StoryId, offset: usize) {
        // The child's old snapshot is not a snapshot of the newly merged parent. Force it
        // out of date even when the old and surviving revision numbers happen to coincide.
        let stale_rev = self.story(to).map_or(0, |st| st.rev.wrapping_sub(1));
        for st in self.stories.values_mut() {
            if st.link.is_some_and(|(parent, _)| parent == from) {
                Arc::make_mut(st).link = Some((to, stale_rev));
            }
        }
        for link in &mut self.hyperlinks {
            if let crate::HyperlinkSource::Text { story, start, end } = &mut link.source
                && *story == from
            {
                *story = to;
                *start = start.saturating_add(offset);
                *end = end.saturating_add(offset);
            }
        }
        for placeholder in &mut self.data_merge.placeholders {
            if let crate::PlaceholderAnchor::Text { story, start, end } = &mut placeholder.anchor
                && *story == from
            {
                *story = to;
                *start = start.saturating_add(offset);
                *end = end.saturating_add(offset);
            }
        }
        if self.settings.primary_story == Some(from) {
            self.settings.primary_story = Some(to);
        }
    }

    /// A linked child is replaced wholesale on update. Only empty, unlinked frame
    /// extensions can join it without losing a link or newly appended body text.
    fn check_thread_links(&self, from: ItemId, to: ItemId, keep_to: bool) -> Result<()> {
        let fs = self.frame_story(from)?;
        let ts = self.frame_story(to)?;
        let first = fs.and_then(|id| self.story(id));
        let second = ts.and_then(|id| self.story(id));
        let incompatible = if keep_to {
            first.is_some_and(|st| st.link.is_some()) || second.is_some_and(|st| st.link.is_some_and(|(parent, _)| Some(parent) == fs))
        } else {
            second.is_some_and(|st| st.link.is_some())
                || first.is_some_and(|st| {
                    st.link.is_some()
                        && second.is_some_and(|incoming| !incoming.text.is_empty() || st.link.is_some_and(|(parent, _)| parent == incoming.id))
                })
        };
        // Removing any ancestor of the survivor would redirect a link back into its own chain.
        let (removed, survivor) = if keep_to { (fs, second) } else { (ts, first) };
        let mut parent = survivor.and_then(|st| st.link);
        let mut absorbs_ancestor = false;
        // Bound the walk even when an existing document already contains a link cycle.
        for _ in 0..self.stories.len() {
            let Some((id, _)) = parent else { break };
            if Some(id) == removed {
                absorbs_ancestor = true;
                break;
            }
            parent = self.story(id).and_then(|st| st.link);
        }
        if incompatible || absorbs_ancestor {
            return Err(DocError::Invalid("unlink the linked story before merging its text".into()));
        }
        Ok(())
    }

    /// Can a user thread `from` → `to`? Both are text frames or empty frames (not lines), on
    /// document spreads or on the same parent spread, not in one story already, and no frame
    /// threads into `to` yet. [`Document::thread`] itself is less strict (import and autoflow).
    pub fn check_thread(&self, from: ItemId, to: ItemId) -> Result<()> {
        if from == to {
            return Err(DocError::Invalid("cannot thread a frame to itself".into()));
        }
        for id in [from, to] {
            let it = self.item(id).ok_or(DocError::NoItem(id))?;
            if !matches!(it.content, Content::Text(_) | Content::Unassigned) || it.shape == Shape::GraphicLine {
                return Err(DocError::Invalid(format!("{id} is not a text frame or an empty frame")));
            }
        }
        let (Some(fl), Some(tl)) = (self.find(from), self.find(to)) else { return Err(DocError::NoItem(to)) };
        let same_place = match (fl.spread, tl.spread) {
            (SpreadRef::Doc(_), SpreadRef::Doc(_)) => true,
            (a, b) => a == b,
        };
        if !same_place {
            return Err(DocError::Invalid("frames on a parent spread thread only to frames on the same parent".into()));
        }
        let story = |id: ItemId| self.item(id).and_then(|i| i.text_frame()).map(|t| t.story);
        if story(from).is_some() && story(from) == story(to) {
            return Err(DocError::Invalid("the frames are already in the same thread".into()));
        }
        if self.prev_frame(to).is_some() {
            return Err(DocError::Invalid(format!("text already flows into {to} from another frame: break that thread first")));
        }
        let fs = self.frame_story(from)?;
        let from_empty = fs.is_none_or(|id| self.story(id).is_some_and(|s| s.text.is_empty() && s.frames == [from]));
        self.check_thread_links(from, to, from_empty && self.frame_story(to)?.is_some())?;
        Ok(())
    }

    /// The story of a frame that can take part in a thread: `Some` for a text frame, `None` for
    /// an empty (unassigned) frame.
    fn frame_story(&self, id: ItemId) -> Result<Option<StoryId>> {
        match &self.item(id).ok_or(DocError::NoItem(id))?.content {
            Content::Text(t) => Ok(Some(t.story)),
            Content::Unassigned => Ok(None),
            _ => Err(DocError::Invalid("can only thread text frames or empty frames".into())),
        }
    }

    /// Make an empty frame a text frame with a new empty story.
    fn give_empty_story(&mut self, id: ItemId) -> Result<StoryId> {
        let sid = StoryId(self.alloc());
        let mut st = Story::new(sid);
        st.frames.push(id);
        let it = self.item_mut(id).ok_or(DocError::NoItem(id))?;
        it.content = Content::Text(TextFrame { story: sid, options: TextFrameOptions::default() });
        self.stories.insert(sid, Arc::new(st));
        Ok(sid)
    }

    /// Break the thread after `frame`: the following frames become a new empty story (the text
    /// stays with the first part and may become overset).
    pub fn unthread_after(&mut self, frame: ItemId) -> Result<()> {
        let sid = self.item(frame).and_then(|i| i.text_frame()).map(|t| t.story).ok_or(DocError::NoItem(frame))?;
        let (tail, vertical, direction) = {
            let st = self.story_mut(sid).ok_or(DocError::NoStory(sid))?;
            let p = st.frames.iter().position(|f| *f == frame).ok_or(DocError::NoItem(frame))?;
            let tail = st.frames.split_off(p + 1);
            st.rev = st.rev.wrapping_add(1);
            (tail, st.vertical, st.direction)
        };
        if tail.is_empty() {
            return Ok(());
        }
        let nid = StoryId(self.alloc());
        let mut ns = Story::new(nid);
        ns.frames = tail.clone();
        ns.vertical = vertical;
        ns.direction = direction;
        self.stories.insert(nid, Arc::new(ns));
        for f in tail {
            if let Some(t) = self.item_mut(f).and_then(Item::text_frame_mut) {
                t.story = nid;
            }
        }
        Ok(())
    }

    /// Take `frame` out of its thread: the frames before and after it join up and keep the text,
    /// `frame` stays on the page with a new empty story.
    pub fn remove_from_thread(&mut self, frame: ItemId) -> Result<()> {
        let sid = self.item(frame).and_then(|i| i.text_frame()).map(|t| t.story).ok_or(DocError::NoItem(frame))?;
        let (vertical, direction) = {
            let st = self.story_mut(sid).ok_or(DocError::NoStory(sid))?;
            if st.frames.len() < 2 {
                return Err(DocError::Invalid(format!("frame {frame} is not threaded to another frame")));
            }
            st.frames.retain(|f| *f != frame);
            st.rev = st.rev.wrapping_add(1);
            (st.vertical, st.direction)
        };
        let nid = StoryId(self.alloc());
        let mut ns = Story::new(nid);
        ns.frames = vec![frame];
        ns.vertical = vertical;
        ns.direction = direction;
        self.stories.insert(nid, Arc::new(ns));
        let t = self.item_mut(frame).and_then(Item::text_frame_mut).ok_or(DocError::NoItem(frame))?;
        t.story = nid;
        Ok(())
    }

    /// Next / previous frame in a thread.
    pub fn next_frame(&self, frame: ItemId) -> Option<ItemId> {
        let st = self.story(self.item(frame)?.text_frame()?.story)?;
        let p = st.frames.iter().position(|f| *f == frame)?;
        st.frames.get(p + 1).copied()
    }
    pub fn prev_frame(&self, frame: ItemId) -> Option<ItemId> {
        let st = self.story(self.item(frame)?.text_frame()?.story)?;
        let p = st.frames.iter().position(|f| *f == frame)?;
        p.checked_sub(1).map(|i| st.frames[i])
    }

    /// Does this text frame set its text vertically (its story is vertical; type on a path stays
    /// along the path)?
    pub fn frame_vertical(&self, item: &Item) -> bool {
        item.text_frame().is_some_and(|t| t.options.path.is_none() && self.story(t.story).is_some_and(|s| s.vertical))
    }

    /// Text space → item inner space for a frame's composed text (identity except for vertical
    /// frames).
    pub fn text_local(&self, item: &Item) -> Affine {
        if self.frame_vertical(item) { crate::item::vertical_text_xf(item.text_area()) } else { Affine::IDENTITY }
    }

    /// Text space → spread-parent space for a frame's composed text (the item transform, with the
    /// quarter turn of a vertical frame).
    pub fn text_xf(&self, item: &Item) -> Affine {
        item.xf * self.text_local(item)
    }

    /// Hit test: the frontmost visible, unlocked-layer item on spread `si` containing `p` (spread space).
    /// Frames hit by area if they have a fill or are text/graphic frames; otherwise near the outline.
    pub fn hit_item(&self, si: usize, p: Point, tol: f64) -> Option<ItemId> {
        let sp = self.spreads.get(si)?;
        let mut items: Vec<&Arc<Item>> = sp.items.iter().collect();
        items.sort_by_key(|i| self.layer_rank(i.layer));
        for it in items.into_iter().rev() {
            let layer = self.layer(it.layer);
            if it.hidden || layer.is_some_and(|l| !l.visible || l.locked) {
                continue;
            }
            if item_hit(it, p, tol) {
                return Some(it.id);
            }
        }
        None
    }

    /// True if `id` itself is locked, sits on a locked layer, or has any locked ancestor group.
    pub fn is_locked(&self, id: ItemId) -> bool {
        let Some(it) = self.item(id) else { return false };
        it.locked || self.layer(it.layer).is_some_and(|l| l.locked) || self.ancestor_locked(id)
    }

    /// True if a group around `id` is locked (the item's own lock and its layer's are not
    /// looked at): the lock `id` inherits.
    pub fn ancestor_locked(&self, id: ItemId) -> bool {
        let chain = self.ancestry_chain(id);
        match chain.split_last() {
            Some((last, parents)) if *last == id => parents.iter().any(|&p| self.item(p).is_some_and(|g| g.locked)),
            _ => false,
        }
    }

    /// Every item id (pre-order) on document spreads.
    pub fn all_items(&self) -> Vec<ItemId> {
        let mut v = Vec::new();
        for sp in &self.spreads {
            for it in &sp.items {
                it.walk(&mut |i| v.push(i.id));
            }
        }
        v
    }

    /// Plain story text with frame/story stats (for agents).
    pub fn story_summary(&self, sid: StoryId) -> Option<serde_json::Value> {
        let st = self.story(sid)?;
        Some(serde_json::json!({
            "id": sid.0,
            "length": st.text.len(),
            "direction": st.direction,
            "paragraphs": st.paras.len(),
            "frames": st.frames.iter().map(|f| f.0).collect::<Vec<_>>(),
            "vertical": st.vertical,
            "text": st.text,
        }))
    }

    /// Replace a story's text and formatting wholesale (agents / import).
    pub fn set_story_text(&mut self, sid: StoryId, text: &str) -> Result<()> {
        let st = self.story_mut(sid).ok_or(DocError::NoStory(sid))?;
        let para = st.paras.first().cloned().map(|p| ParaFormat { table: None, ..p }).unwrap_or_default();
        let fmt = st.chars.first().map(|r| r.format.clone()).unwrap_or_default();
        let n = text.matches('\n').count() + 1;
        st.text = text.to_string();
        st.paras = vec![para; n];
        st.chars = vec![CharRun { len: text.len(), format: fmt }];
        st.tables.clear();
        st.notes.clear();
        st.fix_notes();
        st.endnotes.clear();
        st.fix_endnotes();
        st.editorial.clear();
        st.anchors.clear();
        st.xrefs.clear();
        st.index_refs.clear();
        st.objects.clear();
        st.fix_marks();
        st.rev = st.rev.wrapping_add(1);
        Ok(())
    }
}

/// Does `p` (in the item's parent space) hit `it`?
pub fn item_hit(it: &Item, p: Point, tol: f64) -> bool {
    if let Content::Group { items } = &it.content {
        // A data grid draws its prototype in the origin cell. Other groups use `xf`.
        let inner = it.child_space().inverse() * p;
        if items.iter().any(|c| item_hit(c, inner, tol)) {
            return true;
        }
        // Empty cells stay clickable so frames behind the grid can still be selected.
        // The frame and the cell lines are the grid.
        return it.data_grid.is_some() && grid_outline_hit(it, p, tol);
    }
    let inner = it.xf.inverse() * p;
    let bp = it.path.to_bezpath();
    let area = !it.fill.is_none() || matches!(it.content, Content::Text(_) | Content::Graphic(_)) || it.path.is_closed() && it.shape != Shape::Path;
    if area && it.path.is_closed() && pagenor_geom::hit::fill_contains(&bp, pagenor_geom::FillRule::NonZero, inner) {
        return true;
    }
    pagenor_geom::hit::stroke_contains(&bp, it.stroke.weight.max(1.0), tol, inner)
}

/// The grid rectangle and its cell lines, not the empty interior.
fn grid_outline_hit(it: &Item, p: Point, tol: f64) -> bool {
    let Some(grid) = &it.data_grid else { return false };
    if it.stroke.is_none() {
        return false;
    }
    let inner = it.xf.inverse() * p;
    let width = it.stroke.weight.max(1.0);
    let frame = it.path.to_bezpath();
    if pagenor_geom::hit::stroke_contains(&frame, width, tol, inner) {
        return true;
    }
    let dividers = grid.divider_path(it.inner_bounds());
    pagenor_geom::hit::stroke_contains(&dividers, width, tol, inner)
}

/// The frontmost descendant of `it` under `p` that `want` accepts, looking inside nested groups;
/// `it` itself when nothing inside it matches and `want` accepts it. `p` is in the space `it`
/// sits in, as [`item_hit`] takes it.
///
/// Children are tried front to back and a hidden one is skipped, so of several that `want`
/// accepts the answer is the one in front. Objects `want` rejects do not hide the ones behind
/// them: a text frame under a rectangle in the same group is still found.
pub fn hit_descendant(it: &Item, p: Point, tol: f64, want: &dyn Fn(&Item) -> bool) -> Option<ItemId> {
    hit_descendant_at(it, p, tol, want, 0)
}

fn hit_descendant_at(it: &Item, p: Point, tol: f64, want: &dyn Fn(&Item) -> bool, depth: usize) -> Option<ItemId> {
    if depth >= 64 || !item_hit(it, p, tol) {
        return None;
    }
    let inner = it.xf.inverse() * p;
    let children: Vec<&Arc<Item>> = it.shown_children().collect();
    for c in children.into_iter().rev() {
        if c.hidden {
            continue;
        }
        if let Some(id) = hit_descendant_at(c, inner, tol, want, depth + 1) {
            return Some(id);
        }
    }
    want(it).then_some(it.id)
}
