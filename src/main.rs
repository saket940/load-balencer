use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

struct LoadBalancer {
    backends: Vec<&'static str>,
    counter: AtomicUsize,
}

impl LoadBalancer {
    fn next(&self) -> &str {
        let i = self.counter.fetch_add(1, Ordering::Relaxed);
        self.backends[i % self.backends.len()]
    }
}

fn main() {
    let lb = Arc::new(LoadBalancer {
        backends: vec![
            "https://web-update-alert.onrender.com",
            "https://web-update-alert.onrender.com"
        ],
        counter: AtomicUsize::new(0),
    });

    println!("Next backend: {}", lb.next());
}
