pub fn encode_array(array: Vec<String>) -> Vec<u8> {
    let mut ans = Vec::new();
    let mut length = array.len(); // Get the length of the string
    let mut length_str = length.to_string(); // Convert the length to a string
    let mut length_bytes = length_str.as_bytes(); 
    ans.extend_from_slice(b"*");
    ans.extend_from_slice(length_bytes);
    ans.extend_from_slice(b"\r\n");
    for i in array {
        length = i.len();
        length_str = length.to_string();
        length_bytes = length_str.as_bytes();
        ans.extend_from_slice(b"$");
        ans.extend_from_slice(length_bytes);
        ans.extend_from_slice(b"\r\n");
        ans.extend_from_slice(i.as_bytes());
        ans.extend_from_slice(b"\r\n");
    }
    return ans;
}

pub fn encode_bulk_string(array: Vec<String>) -> Vec<u8> {
    // date should be like this
    // "role:master\r\nmaster_repl_offset:0\r\nmaster_replid:8371b4fb1155b71f4a04d3e1bc3e18c4a990aeeb"

    let mut ans: Vec<u8> = Vec::new();
    let mut result = String::new();
    // append all string with /r/n
    for s in array {
        result.insert_str(result.len(), &s);
        result.insert_str(result.len(), "\r\n");
    }

    // remove last \r\n
    result.pop();
    result.pop();

    // convert string to bulk String
    ans.extend_from_slice(b"$");
    ans.extend_from_slice(result.len().to_string().as_bytes());
    ans.extend_from_slice(b"\r\n");
    ans.extend_from_slice(result.as_bytes());
    ans.extend_from_slice(b"\r\n");

    return ans;
}