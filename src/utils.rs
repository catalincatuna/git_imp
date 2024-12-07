
pub fn extract_after_numeric(input: String, patterns: &[&str]) -> Vec<String> {
    let mut results = Vec::new();
    let mut start_index = 0;

    while let Some(pos) = patterns
        .iter()
        .filter_map(|pattern| input[start_index..].find(pattern).map(|i| (i, *pattern)))
        .min_by_key(|(i, _)| *i) // Find the closest match
    {
        let pattern_pos = start_index + pos.0;
        let pattern = pos.1;
        start_index = pattern_pos + pattern.len();

        let next_part = input[start_index+1..]
            .chars()
            .take_while(|&c| c != '\0')
            .collect::<String>();
        if !next_part.is_empty() {
            results.push(next_part);
        }
    }

    results
}

pub fn extract_filename(input: String) -> String {
    
    let filename = input.chars()
                        .rev()
                        .take_while(|&c| c != '\\')
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect::<String>();
    filename
}