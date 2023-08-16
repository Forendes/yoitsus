pub fn to_time(secs: u64) -> String {
    let sec = (secs % 60) as u8;
    let min = ((secs / 60) % 60) as u8;
    let hrs = secs / 60 / 60;

    if hrs == 0 {
        return format!("{:0>2}:{:0>2}", min, sec);
    }
    format!("{}:{:0>2}:{:0>2}", hrs, min, sec)
}
