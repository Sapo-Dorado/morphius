use lazy_static::lazy_static;
use regex::Regex;
use itertools::Itertools;
use rand::Rng;
use std::cmp;

pub struct Document {
    questions: Vec<Question>,
    layout: Vec<String>
}

pub struct Question {
    text: String
}

pub fn process(input: &str) -> Document {
    lazy_static! {
        static ref QUESTION: Regex = Regex::new(r"\|<q>(.*?)</q>\|").unwrap();
    }
    let questions: Vec<Question> = QUESTION.captures_iter(input).map(|cap| process_question(&cap[1])).collect();
    let layout: Vec<String> = QUESTION.split(input).map(String::from).collect();
    Document{ questions, layout }
}

pub fn generate(doc: &Document, num_results: usize, num_questions: Option<usize>) -> Vec<String> {
    match num_questions {
        Some(num_qs) => {
            let mut rng = rand::thread_rng();
            let tot_qs_in_doc = doc.questions.len();
            let num_permutations = cmp::min(num_qs, tot_qs_in_doc);
            let permutations: Vec<Vec<usize>> = (0..tot_qs_in_doc).permutations(num_permutations).collect();

            (0..num_results).map(|_| gen_form(doc, Some(&permutations[rng.gen_range(0..num_permutations)]))).collect()
        }
        None => (0..num_results).map(|_| gen_form(doc, None)).collect()
    }
}

fn gen_form(doc: &Document, order: Option<&Vec<usize>>) -> String {
    let questions: Vec<String> =  match order {
        Some(ord) => ord.iter().map(|i| gen_question(&doc.questions[*i])).collect(), 
        None => doc.questions.iter().map(|q| gen_question(&q)).collect(),
    };
    doc.layout.iter().interleave(&questions).join("")
}

fn gen_question(question: &Question) -> String {
    question.text.clone()
}

fn process_question(question: &str) -> Question {
    Question{ text: String::from(question) }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORM1: &str = "Beginning|<q>Question 1</q>|Middle|<q>Question 2</q>|End";
    const FORM2: &str = "|<q>1</q>|Middle 1|<q>2</q>|Middle 2|<q>3</q>|";
    const FORM3: &str = "|<q>1</q>||<q>2</q>||<q>3</q>|";

    #[test]
    fn test_process_1() {
        let doc = process(FORM1);
        assert_eq!(doc.layout[0], "Beginning");
        assert_eq!(doc.layout[1], "Middle");
        assert_eq!(doc.layout[2], "End");
        assert_eq!(doc.questions[0].text, "Question 1");
        assert_eq!(doc.questions[1].text, "Question 2");
    }

    #[test]
    fn test_process_2() {
        let doc = process(FORM2);
        assert_eq!(doc.layout, vec!["","Middle 1", "Middle 2",""]);
        assert_eq!(doc.questions[0].text, "1");
        assert_eq!(doc.questions[1].text, "2");
        assert_eq!(doc.questions[2].text, "3");
    }

    #[test]
    fn test_gen_form_original_order() {
        let doc = process(FORM1);
        assert_eq!(gen_form(&doc, None), "BeginningQuestion 1MiddleQuestion 2End");
    }

    #[test]
    fn test_gen_form_different_order() {
        let doc = process(FORM2);
        assert_eq!(gen_form(&doc, Some(&vec![1,2,0])), "2Middle 13Middle 21");
        assert_eq!(gen_form(&doc, Some(&vec![2,1,0])), "3Middle 12Middle 21");
        assert_eq!(gen_form(&doc, Some(&vec![0,1,2])), "1Middle 12Middle 23");
    }

    #[test]
    fn test_generate_no_reorder() {
        let doc = process(FORM3);
        assert_eq!(generate(&doc, 2, None), vec!["123", "123"]);
    }

    #[test]
    fn test_generate_reorder() {
        let doc = process(FORM3);
        let results = generate(&doc, 3, Some(3));
        for result in results {
            assert!(result.contains("1") && result.contains("2") && result.contains("3"));
        }
    }

    #[test]
    fn test_generate_skip_questions() {
        let doc = process(FORM3);
        let results = generate(&doc, 3, Some(1));
        for result in results {
            assert!(!result.contains("1") || !result.contains("2") || !result.contains("3"));
        }
    }
}