#!/usr/bin/env python3
"""
Extract authentic UI resources from .NET WPF PE binaries (Clevo / Acer Control Center 3.0)
Parses the 0xBEEFCACE resource stream directly into valid PNGs, ICOs, and BAML files.
"""

import os
import struct
import sys

def read_7bit(buf, offset):
    val = 0
    shift = 0
    while True:
        b = buf[offset]
        offset += 1
        val |= (b & 0x7F) << shift
        shift += 7
        if (b & 0x80) == 0:
            break
    return val, offset

def extract_dotnet_resources(exe_path, output_dir):
    with open(exe_path, "rb") as f:
        buf = f.read()

    pos = buf.find(b"\xce\xca\xef\xbe")
    if pos == -1:
        print(f"No BEEFCACE signature found in {exe_path}")
        return 0

    magic, hver, skip = struct.unpack("<III", buf[pos : pos + 12])
    p = pos + 12 + skip
    ver, num_res, num_types = struct.unpack("<III", buf[p : p + 12])
    p += 12

    # Skip padding bytes
    while buf[p : p + 3] == b"PAD" or buf[p : p + 1] in (b"P", b"A", b"D"):
        p += 1

    hashes = struct.unpack(f"<{num_res}I", buf[p : p + num_res * 4])
    p += num_res * 4
    name_offsets = struct.unpack(f"<{num_res}I", buf[p : p + num_res * 4])
    p += num_res * 4
    data_sec_offset = struct.unpack("<I", buf[p : p + 4])[0]
    p += 4
    name_sec_pos = p

    # Sequentially read resource names and data offsets
    np = name_sec_pos
    entries = []
    for _ in range(num_res):
        cnt, np = read_7bit(buf, np)
        name = buf[np : np + cnt].decode("utf-16le")
        np += cnt
        doff = int.from_bytes(buf[np : np + 4], "little")
        np += 4
        entries.append((name, doff))

    data_sec_pos = np
    print(f"{exe_path}: {num_res} resources (data_sec @ {hex(data_sec_pos)})")

    saved = 0
    for name, doff in entries:
        dp = data_sec_pos + doff
        type_code = buf[dp]
        dp += 1
        data_len = int.from_bytes(buf[dp : dp + 4], "little")
        dp += 4
        data = buf[dp : dp + data_len]

        out_path = os.path.join(output_dir, name)
        os.makedirs(os.path.dirname(out_path), exist_ok=True)
        with open(out_path, "wb") as out_f:
            out_f.write(data)
        saved += 1

    return saved

if __name__ == "__main__":
    if len(sys.argv) < 3:
        print("Usage: extract_resources.py <path_to_exe> <output_dir>")
        sys.exit(1)
    extract_dotnet_resources(sys.argv[1], sys.argv[2])
