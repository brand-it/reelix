pub fn cast_to_i32(number_string: String) -> i32 {
    number_string.parse::<i32>().unwrap_or_default()
}

pub fn cast_to_u32(number_string: String) -> u32 {
    number_string.parse::<u32>().unwrap_or_default()
}
