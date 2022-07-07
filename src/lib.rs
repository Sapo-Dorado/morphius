use lazy_static::lazy_static;
use regex::Regex;
use itertools::Itertools;
use rand::Rng;
use std::cmp;
use std::collections::{HashSet, HashMap};

pub struct Document {
    pub questions: Vec<Question>,
    pub layout: Vec<String>
}

pub struct Question {
    pub vars: HashSet<String>,
    pub expressions: Vec<Expression>,
    pub layout: Vec<String>,
    pub answer: Option<Answer>
}

pub struct Answer {
    pub expressions: Vec<Expression>,
    pub layout: Vec<String>
}

struct Content {
    vars: HashSet<String>,
    expressions: Vec<Expression>,
    layout: Vec<String>
}

pub struct Expression {
    pub expression: Vec<ExpComp>
}

pub struct Test {
    pub content: String,
    pub answers: String
}

pub enum ExpComp {
    Var(String),
    Other(String)
}

pub fn process(input: &str) -> Document {
    lazy_static! {
        static ref QUESTION: Regex = Regex::new(r"(?s)\|<q>(.*?)</q>\|").unwrap();
    }
    let questions: Vec<Question> = QUESTION.captures_iter(input).map(|cap| process_question(&cap[1], None)).collect();
    let layout: Vec<String> = QUESTION.split(input).map(String::from).collect();
    Document{ questions, layout }
}

pub fn process_with_answers(input: &str) -> Document {
    lazy_static! {
        static ref QUESTION: Regex = Regex::new(r"(?s)\|<q>(.*?)</q>\|\s*\|<a>(.*?)</a>\|").unwrap();
    }
    let mut questions: Vec<Question> = Vec::new();
    for cap in QUESTION.captures_iter(input) {
        questions.push(process_question(&cap[1], Some(process_answer(&cap[2]))));
    }
    let layout: Vec<String> = QUESTION.split(input).map(String::from).collect();
    Document{ questions, layout }
}

pub fn generate(doc: &Document, num_results: usize, num_questions: Option<usize>) -> Vec<Test> {
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

fn gen_form(doc: &Document, order: Option<&Vec<usize>>) -> Test {
    let mut questions: Vec<String> = Vec::new();
    let mut answers: Vec<String> = Vec::new();
    match order {
        Some(ord) => {
            for i in ord.iter() {
                let (content, answer) = gen_question_text(&doc.questions[*i]);
                questions.push(content);
                answers.push(answer);
            }
        },
        None => {
            for q in doc.questions.iter() {
                let (content, answer) = gen_question_text(&q);
                questions.push(content);
                answers.push(answer);
                ()
            }
        }
    };
    Test { content: doc.layout.iter().interleave(&questions).join(""), answers: doc.layout.iter().interleave(&answers).join("") }
}

fn gen_question_text(question: &Question) -> (String, String) {
    let mut rng = rand::thread_rng();
    let mut scope:HashMap<&str,i64> = HashMap::new();
    for var in question.vars.iter() {
        scope.insert(&var[..], rng.gen_range(0..100));
    }


    let content = question.layout.iter().interleave(&question.expressions.iter().map(|exp| gen_expression_text(exp, &scope)).collect::<Vec<String>>()).join("");

    let answer: String = match &question.answer {
        Some(answer) => answer.layout.iter().interleave(&answer.expressions.iter().map(|exp| gen_expression_text(exp, &scope)).collect::<Vec<String>>()).join(""),
        None => String::from("No Answers Provided")
    };

    (content, answer)
}

fn gen_expression_text(expression: &Expression, scope: &HashMap<&str,i64>) -> String {
    let expr = expression.expression.iter().map(|exp_cmp| {
        match exp_cmp {
            ExpComp::Var(var_name) => scope.get(&var_name[..]).unwrap().to_string(),
            ExpComp::Other(text) => text.clone()
        }
    })
    .join("");
    match mexprp::eval::<f64>(&expr).unwrap() {
        mexprp::Answer::Single(num) => num.to_string(),
        mexprp::Answer::Multiple(_) => panic!("Unsupported math")
    }
}

fn process_question(question: &str, answer: Option<Answer>) -> Question {
    let content = get_content(question);
    Question { vars: content.vars, expressions: content.expressions, layout: content.layout, answer }
}

fn process_answer(answer: &str) -> Answer {
    let content = get_content(answer);
    Answer { expressions: content.expressions, layout: content.layout }
}

fn get_content(text: &str) -> Content {
    lazy_static! {
        static ref EXP: Regex = Regex::new(r"\|<e>(.*?)</e>\|").unwrap();
    }
    let mut vars: HashSet<String> = HashSet::new();
    let expressions: Vec<Expression> = EXP.captures_iter(text).map(|cap| process_expression(&cap[1], &mut vars)).collect();
    let layout: Vec<String> = EXP.split(text).map(String::from).collect();
    Content{ vars, expressions, layout }
}

fn process_expression(expression: &str, vars: &mut HashSet<String>) -> Expression {
    lazy_static! {
        static ref VAR: Regex = Regex::new(r"[[:alpha:]][[:word:]]*").unwrap();
    }
    let mut vars_list: Vec<ExpComp> = Vec::new();
    for cap in VAR.captures_iter(expression) {
        vars.insert(String::from(&cap[0]));
        vars_list.push(ExpComp::Var(String::from(&cap[0])));
    }
    Expression { expression: VAR.split(expression).map(|text| ExpComp::Other(String::from(text))).interleave(vars_list).collect() }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FORM1: &str = "Beginning|<q>Question 1</q>|Middle|<q>Question 2</q>|End";
    const FORM2: &str = "|<q>1</q>|Middle 1|<q>2</q>|Middle 2|<q>3</q>|";
    const FORM3: &str = "|<q>1</q>||<q>2</q>||<q>3</q>|";
    const FORM4: &str = "|<q>1</q>||<a>3</a>|";
    const FORM5: &str = "|<q>1</q>|\n\n          \t\t|<a>3</a>|";


    #[test]
    fn test_process_1() {
        let doc = process(FORM1);
        assert_eq!(doc.layout[0], "Beginning");
        assert_eq!(doc.layout[1], "Middle");
        assert_eq!(doc.layout[2], "End");
        assert_eq!(gen_question_text(&doc.questions[0]).0, "Question 1");
        assert_eq!(gen_question_text(&doc.questions[1]).0, "Question 2");
    }

    #[test]
    fn test_process_2() {
        let doc = process(FORM2);
        assert_eq!(doc.layout, vec!["","Middle 1", "Middle 2",""]);
        assert_eq!(gen_question_text(&doc.questions[0]).0, "1");
        assert_eq!(gen_question_text(&doc.questions[1]).0, "2");
        assert_eq!(gen_question_text(&doc.questions[2]).0, "3");
    }

    #[test]
    fn test_gen_form_original_order() {
        let doc = process(FORM1);
        assert_eq!(gen_form(&doc, None).content, "BeginningQuestion 1MiddleQuestion 2End");
    }

    #[test]
    fn test_gen_form_different_order() {
        let doc = process(FORM2);
        assert_eq!(gen_form(&doc, Some(&vec![1,2,0])).content, "2Middle 13Middle 21");
        assert_eq!(gen_form(&doc, Some(&vec![2,1,0])).content, "3Middle 12Middle 21");
        assert_eq!(gen_form(&doc, Some(&vec![0,1,2])).content, "1Middle 12Middle 23");
    }

    #[test]
    fn test_generate_no_reorder() {
        let doc = process(FORM3);
        assert_eq!(generate(&doc, 2, None)[0].content, "123");
        assert_eq!(generate(&doc, 2, None)[1].content, "123");
    }

    #[test]
    fn test_generate_reorder() {
        let doc = process(FORM3);
        let results = generate(&doc, 3, Some(3));
        for result in results {
            assert!(result.content.contains("1") && result.content.contains("2") && result.content.contains("3"));
        }
    }

    #[test]
    fn test_generate_skip_questions() {
        let doc = process(FORM3);
        let results = generate(&doc, 3, Some(1));
        for result in results {
            assert!(!result.content.contains("1") || !result.content.contains("2") || !result.content.contains("3"));
        }
    }

    #[test]
    fn test_generate_var() {
        let doc = process("|<q>|<e>a</e>|</q>|");
        let result = generate(&doc, 3, Some(1));
        let num_re = Regex::new(r"^[[:digit:]]+$").unwrap();
        assert!(num_re.is_match(&result[0].content));
    }

    #[test]
    fn test_generate_var_math() {
        let doc = process("|<q>|<e>(a+b)-c</e>|</q>|");
        let result = generate(&doc, 3, Some(1));
        let num_re = Regex::new(r"^-?[[:digit:]]+$").unwrap();
        assert!(num_re.is_match(&result[0].content));
    }

    #[test]
    fn test_process_with_anwer() {
        let doc1 = process_with_answers(FORM4);
        match doc1.questions[0].answer {
            Some(_) => assert!(true),
            None => assert!(false)
        }

        let doc2 = process(FORM4);
        match doc2.questions[0].answer {
            Some(_) => assert!(false),
            None => assert!(true)
        }
    }

    #[test]
    fn test_process_with_anwer_newlines_are_ok() {
        let doc = process_with_answers(FORM5);
        match doc.questions[0].answer {
            Some(_) => assert!(true),
            None => assert!(false)
        }
    }

    #[test]
    fn test_answer_generated_correctly() {
        let doc = process_with_answers("|<q>|<e>a</e>|</q>||<a>|<e>a</e>|</a>|");
        for result in generate(&doc, 3, Some(1)) {
            assert_eq!(result.content, result.answers);
        }
        
    }
}