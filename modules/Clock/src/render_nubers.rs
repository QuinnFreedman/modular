pub fn u8_to_str_b10<'a>(buffer: &'a mut [u8; 3], mut n: u8) -> &'a [u8] {
    const POWERS: [u8; 3] = [100, 10, 1];
    let mut cursor = 2u8;
    for i in 0u8..3u8 {
        let power_value = POWERS[i as usize];
        let digit = n / power_value;
        let remainder = n % power_value;
        buffer[i as usize] = ('0' as u8) + digit;
        if digit != 0 && cursor == 2u8 {
            cursor = i
        }
        n = remainder;
    }
    &buffer[(cursor as usize)..]
}

pub fn i8_to_str_b10<'a>(buffer: &'a mut [u8; 4], n: i8) -> &'a [u8] {
    if n == 0 {
        buffer[0] = '0' as u8;
        return &buffer[..1];
    }

    let sign = if n < 0 { '-' } else { '+' };

    let mut n: u8 = n.abs() as u8;
    const POWERS: [u8; 3] = [100, 10, 1];
    let mut cursor = 0xffu8;
    for i in 0u8..3u8 {
        let power_value = POWERS[i as usize];
        let digit = n / power_value;
        let remainder = n % power_value;
        buffer[(i + 1) as usize] = ('0' as u8) + digit;
        if digit != 0 && cursor == 0xffu8 {
            cursor = i
        }
        n = remainder;
    }
    buffer[cursor as usize] = sign as u8;
    &buffer[(cursor as usize)..]
}
