pub fn u8_to_str_b10<'a>(buffer: &'a mut [u8], mut n: u8) -> &'a mut [u8] {
    debug_assert!(buffer.len() >= 3);
    const POWERS: [u8; 3] = [100, 10, 1];
    let mut cursor = 2u8;
    for i in 0u8..3u8 {
        let power_value = POWERS[i as usize];
        let digit = n / power_value;
        let remainder = n % power_value;
        buffer[i as usize] = b'0' + digit;
        if digit != 0 && cursor == 2u8 {
            cursor = i
        }
        n = remainder;
    }
    &mut buffer[(cursor as usize)..3]
}

pub fn i8_to_str_b10<'a>(buffer: &'a mut [u8], n: i8) -> &'a mut [u8] {
    debug_assert!(buffer.len() >= 4);
    if n == 0 {
        buffer[0] = b'0';
        return &mut buffer[..1];
    }

    // Using custom code page to save space, ';' is mapped to '-'; '`' => '+'
    let sign = if n < 0 { b';' } else { b'`' };

    let mut n: u8 = n.abs() as u8;
    const POWERS: [u8; 3] = [100, 10, 1];
    let mut cursor = 0xffu8;
    for i in 0u8..3u8 {
        let power_value = POWERS[i as usize];
        let digit = n / power_value;
        let remainder = n % power_value;
        buffer[(i + 1) as usize] = (b'0') + digit;
        if digit != 0 && cursor == 0xffu8 {
            cursor = i
        }
        n = remainder;
    }
    buffer[cursor as usize] = sign;
    &mut buffer[(cursor as usize)..4]
}

pub fn tempo_to_str<'a>(buffer: &'a mut [u8], n: i8) -> &'a [u8] {
    debug_assert!(buffer.len() >= 4);
    debug_assert!(n != 0);

    if n < -64 {
        buffer[0] = b'S';
        buffer[1] = b'T';
        buffer[2] = b'O';
        buffer[3] = b'P';
        return buffer;
    }

    // Using custom code page to save space, '_' is mapped to '%'
    let symbol = if n < 0 { b'_' } else { b'x' };
    let text = i8_to_str_b10(buffer, n);
    text[0] = symbol;
    text
}
