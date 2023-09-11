pub fn euclidean_distance(a: Vec<f64>, b: Vec<f64>) -> f64 {
    assert_eq!(a.len(), b.len());
    a.iter().zip(b).fold(0.0, |sum, (a, b)| sum + (a - b).powi(2)).sqrt()
}
