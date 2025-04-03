pub fn cast_to_i32(number_string: String) -> i32 {
    match number_string.parse::<i32>() {
        Ok(num) => num,
        Err(_e) => 0,
    }
}

pub fn cast_to_u32(number_string: String) -> u32 {
    match number_string.parse::<u32>() {
        Ok(num) => num,
        Err(_e) => 0,
    }
}
