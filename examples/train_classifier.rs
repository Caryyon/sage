// Train and test the pattern classifier

use sage::learning::pattern_classifier::PatternClassifier;

fn main() {
    println!("=== Pattern Classifier Training ===\n");

    let mut classifier = PatternClassifier::new();

    println!("Bootstrapping network from rule-based detectors...");
    classifier.bootstrap_from_rules(1000);

    println!("\nEvaluating accuracy on test set...");
    let accuracy = classifier.evaluate_accuracy(200);
    println!("Accuracy: {:.2}%", accuracy * 100.0);

    println!("\nTraining complete!");
}
