#[allow(dead_code)]
pub fn euclidean_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(a, b)| (a - b).powi(2)).sum::<f64>().sqrt()
}

#[allow(dead_code)]
pub fn cosine_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());

    let ab: f64 = a.iter().zip(b).map(|(a, b)| a * b).sum();
    let aa: f64 = a.iter().map(|a| a * a).sum();
    let bb: f64 = b.iter().map(|b| b * b).sum();

    1.0 - (ab / (aa.sqrt() * bb.sqrt()))
}

#[allow(dead_code)]
pub fn hamming_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    1.0 - a.iter().zip(b).map(|(a, b)| if a == b { 1.0 } else { 0.0 }).sum::<f64>()
        / (a.len() as f64)
}

#[allow(dead_code)]
pub fn manhattan_distance(a: &Vec<f64>, b: &Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).map(|(a, b)| (a - b).abs()).sum()
}
