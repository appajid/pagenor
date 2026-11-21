#!/usr/bin/env python3
"""Original synthetic compound document: three pages and an original English/8-bit text frame in two PAGE containers.

No user publication, font or Adobe-produced fixture. CFB layout follows Microsoft's
MS-CFB specification; record fields follow the MPL helper's documented parser.
This small input tests text transport and all-container traversal end to end;
it is not an oracle for real-document typography or image recovery.
"""
from pathlib import Path
import struct
import json
import sys


TEXT = b'Pagenor PageMaker fixture: English 123 and caf\xe9.'


def document():
    free, end = 0xffffffff, 0xfffffffe
    stream = bytearray(8192)
    struct.pack_into('<H', stream, 6, 0x99ff)
    # Sequence numbers are the zero-based table positions, including both PAGE containers.
    records = [(0x18, 1, 256), (5, 1, 3000), (5, 2, 3472), (0x13, 1, 5000),
               (0x19, 1, 5200), (0x1a, 1, 5500), (0x1b, len(TEXT), 5600),
               (0x1c, 1, 5700), (0x0b, 1, 5800)]
    struct.pack_into('<HI', stream, 0x2e, len(records), 64)
    for i, (kind, count, offset) in enumerate(records):
        struct.pack_into('<BBHI8x', stream, 64 + i * 16, 0, kind, count, offset)
    struct.pack_into('<hhhh', stream, 256 + 0x36, 0, 0, 12240, 15840)
    for offset in [3000, 3472, 3944]:
        struct.pack_into('<H', stream, offset + 2, 0xffff)  # no shape container
    struct.pack_into('<H', stream, 3000 + 2, 4)  # first page references shape sequence 4
    stream[5000:5006] = b'Arial\0'
    stream[5200] = 1  # a text rectangle, 1/2 inch from the page edges
    struct.pack_into('<hhhh', stream, 5206, -5400, -7200, 3960, -5760)
    struct.pack_into('<II', stream, 5200 + 28, 0, 77)  # identity transform, synthetic story id
    struct.pack_into('<6H', stream, 5500, 0, 0, 6, 7, 8, 0)  # text, character and paragraph sequences
    struct.pack_into('<I', stream, 5500 + 32, 77)
    stream[5600:5600 + len(TEXT)] = TEXT
    struct.pack_into('<HHH', stream, 5700, len(TEXT), 0, 120)  # one 12 pt Arial run
    struct.pack_into('<H', stream, 5700 + 28, 100)  # full tint
    struct.pack_into('<H', stream, 5800, len(TEXT))  # one paragraph
    header = bytearray(512)
    header[:8] = bytes.fromhex('d0cf11e0a1b11ae1')
    struct.pack_into('<HHHHH', header, 24, 0x3e, 3, 0xfffe, 9, 6)
    struct.pack_into('<IIIIIIIII', header, 40, 0, 1, 1, 0, 4096, end, 0, end, 0)
    struct.pack_into('<109I', header, 76, 0, *([free] * 108))
    fat = bytearray(struct.pack('<128I', *([free] * 128)))
    struct.pack_into('<II', fat, 0, 0xfffffffd, end)
    for sector in range(2, 18):
        struct.pack_into('<I', fat, sector * 4, sector + 1 if sector < 17 else end)
    directory = bytearray(512)
    for index, name, kind, child, start, size in [(0, 'Root Entry', 5, 1, end, 0), (1, 'PageMaker', 2, free, 2, len(stream))]:
        offset = index * 128
        encoded = (name + '\0').encode('utf-16le')
        directory[offset:offset + len(encoded)] = encoded
        struct.pack_into('<HBBIII', directory, offset + 64, len(encoded), kind, 1, free, free, child)
        struct.pack_into('<IQ', directory, offset + 116, start, size)
    return bytes(header + fat + directory + stream)


if __name__ == '__main__':
    path = Path(sys.argv[1])
    path.write_bytes(document())
    path.with_suffix('.expected.json').write_text(json.dumps({'pages': 3, 'text': TEXT.decode('cp1252')}))
