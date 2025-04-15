use std::{collections::HashMap, env, fs};
use itertools::Itertools;
use pyo3::{py_run, types::{IntoPyDict, PyAnyMethods}, PyErr, Python};
use stop_words::LANGUAGE;
use once_cell::sync::Lazy;

static STOP_WORDS: Lazy<Vec<String>> = Lazy::new(|| stop_words::get(LANGUAGE::Russian));
static PUNCTUATION: &str = r##"
    !"#$%&'()*+,-./:;<=>?@[\]^_`{|}~
"##;

fn valid_token(token: &String) -> bool {
    token != " " &&
    token.chars().all(|x| !PUNCTUATION.contains(x)) && 
    !STOP_WORDS.contains(token)
}

fn read_file(path: &str) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(String::from)
        .collect()
}

fn get_idf() -> HashMap<String, f64> {
    HashMap::<String, f64>::from_iter(
        read_file("../artifacts/idf.md")
            .iter()
            .skip(2)
            .map(|x| {
                let split = x.split('|').collect::<Vec<&str>>();
                let token = split
                    .get(1)
                    .unwrap()
                    .trim()
                    .to_string();
                let value = split
                    .get(2)
                    .unwrap()
                    .trim()
                    .parse::<f64>()
                    .unwrap();
                (token, value)
            })
    )
}

fn get_tf_idf() -> HashMap<String, Vec<f64>> {
    HashMap::<String, Vec<f64>>::from_iter(
        read_file("../artifacts/tf-idf.md")
            .iter()
            .skip(2)
            .map(|x| {
                let split = x.split('|').collect::<Vec<&str>>();
                let token = split
                    .get(1)
                    .unwrap()
                    .trim()
                    .to_string();
                let values = Vec::<f64>::from_iter(
                    split
                        .iter()
                        .skip(2)
                        .filter(|x| **x != "")
                        .map(|x| {
                            x
                                .trim()
                                .parse::<f64>()
                                .unwrap()
                        })
                        .collect::<Vec<f64>>()
                );
                (token, values)
            })
    )
}

fn get_index() -> Vec<String> {
    read_file("../artifacts/index.txt")
}

fn lemmatize(input: String) -> Result<Vec<String>, PyErr> {
    Python::with_gil::<_, Result<Vec<String>, PyErr>>(|py| {
        let code = r#"
            from pymystem3 import Mystem
            stemmer = Mystem()
            tokens = stemmer.lemmatize(input)
        "#;
    
        let locals = [("input", input)].into_py_dict(py)?;
        
        py_run!(py, *locals, code);
    
        locals.get_item("tokens").unwrap().extract::<Vec<String>>()
    })
}

fn get_document_vector_length(document_vector: &HashMap<String, f64>) -> f64 {
    document_vector
        .iter()
        .fold(0.0, |acc, (_, v)| acc + v * v)
        .sqrt()
}

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let input = match args.get(1) {
        Some(x) => x
            .to_lowercase()
            .trim()
            .to_string(),
        None => panic!("input is empty")
    };

    let input_tokens = match lemmatize(input) {
        Ok(res) => res
            .iter()
            .map(|x| x.to_string())
            .filter(valid_token)
            .collect(),
        Err(_) => vec![]
    };

    if input_tokens.len() == 0 {
        return;
    }

    let input_frequences = HashMap::<String, f64>::from_iter(
        input_tokens
            .iter()
            .counts()
            .iter()
            .map(|x| (x.0.to_string(), (*x.1 as f64 / input_tokens.len() as f64)))
    );

    let idf = get_idf();
    let tf_idf = get_tf_idf();
    let index = get_index();

    let input_vector = HashMap::<String, f64>::from_iter(
        idf
            .iter()
            .map(|x| (x.0.to_string(), input_frequences.get(x.0).unwrap_or(&0.0) * x.1))
    );

    let input_vector_length = get_document_vector_length(&input_vector);

    let mut document_index = 0;

    let mut result = HashMap::new();

    loop {
        if document_index == index.len() {
            break;
        }

        let document_vector = HashMap::<String, f64>::from_iter(
            tf_idf
                .iter()
                .map(|x| (x.0.to_string(), *x.1.get(document_index).unwrap()))
        );

        let scalar_product = document_vector
            .iter()
            .fold(0.0, |acc, (k, v)| acc + v * input_vector.get(k).unwrap());

        let document_vector_length = get_document_vector_length(&document_vector);
        
        let cosine_symmetry = scalar_product / ((document_vector_length * input_vector_length));

        result.insert(document_index, cosine_symmetry);

        document_index += 1;
    }

    let documents = result
        .iter()
        .sorted_by(|a, b| b.1.partial_cmp(a.1).unwrap());

    for (k, v) in documents {
        println!("{} {}", v, index.get(*k).unwrap())
    }
}
