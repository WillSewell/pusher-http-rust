use regex::Regex;


pub fn validate_channels<'a>(channels: &Vec<String>)-> Result<bool, &'a str>{
  if channels.len() > 10 { return Err("Cannot trigger on more than 10 channels") }

  let channel_regex = Regex::new(r"^[-a-zA-Z0-9_=@,.;]+$").unwrap(); // how to make this global?

  for channel in channels {
    if channel.len() > 200 { return Err("Channel names must be under 200 characters") }
    if !channel_regex.is_match(channel) {return Err("Channels must be formatted as such: ^[-a-zA-Z0-9_=@,.;]+$")}
  } 
  Ok(true)
}