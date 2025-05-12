use std::{
    io::Read,
    path::{self, Path},
};

use thiserror::Error;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
/// Errors that can occur when loading a corpus.
#[derive(Debug, Error)]
pub enum FuzzySearchError {
    /// Unable to open the corpus file.
    #[error("Unable to open corpus file")]
    UnableToOpenCorpusFile,

    /// Unable to read the corpus file to a string.
    #[error("Unable to read corpus file to string")]
    UnableToReadCorpusFileToString,
}

/// Loads a corpus from a file.
///
/// The file is expected to be newline-separated lines of text.
/// Each line is loaded as a separate string.
///
/// # Errors
///
/// Returns an error if the corpus file is unable to be opened or read.
fn load_corpus<P: AsRef<Path>>(path: P) -> Result<Vec<String>, FuzzySearchError> {
    // Open corpus file
    let mut corpus_file =
        std::fs::File::open(path).map_err(|_| FuzzySearchError::UnableToOpenCorpusFile)?;

    // Read corpus file to string
    let mut corpus = String::new();
    corpus_file
        .read_to_string(&mut corpus)
        .map_err(|_| FuzzySearchError::UnableToReadCorpusFileToString)?;

    // Split corpus into lines, creating a vector of strings
    Ok(corpus.split("\n").map(|s| s.to_string()).collect())
}

/// Finds the string in the corpus closest to the given string.
///
/// The distance is calculated as a normalized value between 0 and 1.
/// The correctness of a string is 1 - distance, so that correctness ranges
/// from 0 (completely incorrect) to 1 (completely correct).
fn find_closest_str<'a>(arg: &'a str, reference_strs: &'a [String]) -> String {
    let mut closest_str = &reference_strs[0];
    let mut closest_distance = arg.len() as f64;

    // Define a closure to calculate the distance between two strings
    // The distance between two strings is the number of characters that must be
    // changed to transform one string into another.
    // The distance is calculated as a normalized value between 0 and 1.
    let distance = |a: &str, b: &str| -> f64 {
        let mut distance = 0;

        // Calculate the absolute difference in length between a and b
        // Add this to the distance, because the distance is calculated as a
        // normalized value between 0 and 1.
        // Case 0: a is longer than b, so add nonzero distance using abs_diff.
        // Case 1: b is longer than a, so add nonzero distance using abs_diff.
        // Case 2: a and b are the same length, so add zero distance
        let a_len = a.len();
        let b_len = b.len();
        distance += a_len.abs_diff(b_len);

        for (a_char, b_char) in a.chars().zip(b.chars()) {
            // If the characters are not equal, add one to the distance.
            if a_char != b_char {
                distance += 1;
            }
        }

        // Normalize the distance by dividing by the length of the argument string.
        let normalized_distance = distance as f64 / a_len as f64;
        normalized_distance
    };

    // Create a vector to store the correctness of each string in the corpus.
    let mut correctness = vec![0f64; reference_strs.len()];

    for (idx, reference_str) in reference_strs.iter().enumerate() {
        // Calculate the distance between the argument string and the current corpus string.
        let distance = distance(arg, reference_str);

        // Store the correctness of the current corpus string.
        correctness[idx] = 1.0 - (distance as f64 / arg.len() as f64);

        // Update the closest string if the current corpus string is closer than the current closest string.
        if distance < closest_distance {
            closest_str = reference_str;
            closest_distance = distance;
        }
    }

    // PERFORMANCE: Can be optimized to avoid cloning?
    closest_str.clone()
}

pub struct FuzzySearcher {
    corpus: Vec<String>,
}

impl FuzzySearcher {
    /// Creates a new `FuzzySearcher` instance by loading a corpus from a specified file path.
    ///
    /// The corpus is expected to be a newline-separated file containing words or strings.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to the file containing the corpus.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `FuzzySearcher` instance if successful, or a `FuzzySearchError` if an error occurs.
    ///
    /// # Errors
    ///
    /// Returns `FuzzySearchError` if the corpus file cannot be opened or read.

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, FuzzySearchError> {
        let corpus_path = path::Path::new("corpus/words.txt");
        let corpus = load_corpus(corpus_path)?;

        Ok(Self { corpus })
    }

    /// Searches the corpus for the string closest to the given argument string.
    ///
    /// Returns the closest string from the corpus.
    pub fn search(&self, arg: &str) -> String {
        find_closest_str(arg, &self.corpus)
    }
}
