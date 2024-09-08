pub fn get_amount_to_emit(amount_to_stream: &u128, duration: &u128, time_passed: &u128) -> u64 {
    ((time_passed * amount_to_stream) / duration) as u64
}
