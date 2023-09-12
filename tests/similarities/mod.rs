pub fn euclidean_distance(a: Vec<f64>, b: Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(&b).fold(0.0, |sum, (a, b)| sum + (a - b).powi(2)).sqrt()
}

pub fn hamming_distance(a: Vec<f64>, b: Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    1.0 - a.iter().zip(&b).fold(0.0, |sum, (a, b)| sum + (if a == b { 1.0 } else { 0.0 }))
        / (a.len() as f64)
}

pub fn manhattan_distance(a: Vec<f64>, b: Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(&b).fold(0.0, |sum, (a, b)| sum + (a - b).abs())
}
