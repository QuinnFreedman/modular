import sys
from math import ceil

def render_letter(bytes, width, height):
    num_rows = ceil(height / 8) * 8
    # rows = [[]] * num_rows
    rows = [[] for _ in range(num_rows)]
    cursor = 0
    for col in range(width):
        for row_offset in range(num_rows // 8):
            byte = bytes[cursor]
            cursor += 1
            # print(bin(byte))

            for bit_offset in range(8):
                bit = bool((byte >> bit_offset) & 1)
                rows[row_offset * 8 + bit_offset].append(bit)

    result = [[] for _ in range(num_rows//2)]
    for row_index in range(0, len(rows), 2):
        row1 = rows[row_index]
        row2 = rows[row_index + 1]
        symbols = {
            (True, True): '\u2588',
            (True, False): '\u2580',
            (False, True): '\u2584',
            (False, False): ' ',
        }
        for i in range(width):
            symbol = symbols[(row1[i], row2[i])]
            result[row_index//2].append(symbol)

    return ["".join(row) for row in result]


def print_grid(letters):
    ROW_WIDTH = 6
    letter_height = len(letters[0][1])
    letter_width = len(letters[0][1][0])
    print('\u2591' * ((letter_width + 1) * ROW_WIDTH + 1))
    for i in range(0, len(letters), ROW_WIDTH):
        row = letters[i:i+ROW_WIDTH]
        for pixel_row in range(letter_height):
            print('\u2591', end="")
            for offset, letter in row:
                if pixel_row == 0:
                    offset_str = str(offset)
                    row_str = offset_str + letter[pixel_row][len(offset_str):]
                else:
                    row_str = letter[pixel_row]
                print(row_str, end="")
                print('\u2591', end="")
            print()
        print('\u2591' * ((letter_width + 1) * len(row) + 1))
        
with open(sys.argv[1], 'rb') as f:
    data = f.read()
    width = int(sys.argv[2])
    height = int(sys.argv[3])
    offset = 0
    bytes_per_glyph = width * ceil(height/8)
    num_glyphs = len(data) // bytes_per_glyph

    print(f"file size: {len(data)} bytes")
    print(f"glyphs: {num_glyphs}")
    print(f"glyph size: {width}x{height}")
    print(f"bytes per glyph: {bytes_per_glyph}")
    
    letters = []
    for i in range(num_glyphs):
        offset = i * bytes_per_glyph
        letters.append((offset, render_letter(data[offset:offset+bytes_per_glyph], width, height)))

    print_grid(letters)
