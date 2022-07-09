//! # morphius
//!
//! `morphius` allows users to randomize the order and content of documents
//! which can be used by teachers for generating tests with questions in a 
//! different order or with different numbers for each student. If answers
//! are provided, morphius will generate an answer key for each final test to
//! make grading easier.
//! 
//! # Format
//! `morphius` processes content in a specific format described below:
//! 
//! ##### Questions
//! A question in `morphius` is indicated in the following format:
//! 
//! `"|<q>Question Content</q>|"` The content inside the `"|<q>"` and the `"</q>|"`
//! will be rearranged if desired to create a different order of questions on different 
//! tests while the rest of the content in the file remains in the same place. This means
//! that if question numbers are included in the template, those should be placed outside
//! of the question itself in order to retain the correct numbering when questions are 
//! rearranged.
//! 
//! ##### Expressions
//! Expressions are used to add randomness to questions:
//! 
//! `|<e>a+b</e>|` is an example expression. Expressions must be placed inside of a question
//! in order to be processed. A variable in an expression is any identifier that starts with 
//! a letter followed by a sequence of characters that can contain letters, numbers or underscores.
//! Variables do not have to be declared and will default to be an integer between 0 and 99 when generated.
//! Math is allowed in these expressions in order to create specific relationships between numbers in the final
//! question. Math is supported using the `mexprp` crate. The scope of variables is the question, so you can have
//! the same variable names in different questions and they will likely have different values (unless they randomly
//! end up being the same). If you want more fine tuned control of the range of possible values, you can declare
//! the variable.
//! 
//! ##### Variable Declarations
//! A variable can be declared anywhere in the question in the following format:
//! 
//! `|<v>var_name: type = [min,max]</v>|` where var_name is the name of your variable, type is either int or real, and min and max
//! are integers representing the lower and upper bounds respectively of the value of your variable. An example declaration would be
//! `|<v>a: int = [0,99]</v>|` This is the declaration assumed for any variable without a declaration, so including this exact
//! declaration in your code would be unecessary.
//! 
//! ##### Answers
//! 
//! Answers are used to generate an answer key for each test. Answers should be included for every question when using
//! `process_with_answers` They should be in the format `|<a>Answer</a>|` and should appear right after the question. Variables
//! in answers have the same scope as their coresponding question so you can use expressions in your answers to calculate the
//! answer in terms of the randomly generated variables in your question.
//! 
//! 
//! # Examples
//! Here is a simple example, you can find more example templates in the `examples` folder of the GitHub repository.
//! ```
//! let template = "
//! |<q>This is a single question test. You must calculate the sum of two random numbers.
//! What is |<e>a</e>| + |<e>b</e>|?</q>|
//! |<a>The answer to this question is: |<e>a+b</e>|</a>|";
//! 
//! let doc = morphius::process_with_answers(template);
//! let tests = morphius::generate(&doc, 5, Some(1));
//! 
//! //Prints out the first test
//! println!("{}", tests[0].content);
//! 
//! //Prints out the answer key for the first test
//! println!("{}", tests[0].answers);
//! 
//! ```
//! 
//! 
use lazy_static::lazy_static;
use regex::Regex;
use itertools::Itertools;
use rand::Rng;
use std::cmp;
use std::collections::{HashSet, HashMap};

/// A Document is a template to be used to generate filled out tests
pub struct Document {
    ///This is a list of the questions in the Document, in the order provided
    pub questions: Vec<Question>,
    ///This is a list of the other content in the Document that should stay in the same place when the questions move
    pub layout: Vec<String>
}

///A Test is generated from a Document and is ready for use
pub struct Test {
    ///A String representing the contents of the Test
    pub content: String,
    ///A String representing the answers of a Test
    pub answers: String
}

///A Question is an object representing a question
pub struct Question {
    ///This is a list of variables used in the question
    pub vars: HashSet<Var>,
    ///This is a list of expressions that need to be evaluated when generating the question text
    pub expressions: Vec<Expression>,
    ///This is a list of the other content in the question that does not need to be evaluated
    pub layout: Vec<String>,
    ///This is either the Answer to the question, if provided or None
    pub answer: Option<Answer>
}

///An Answer is the answer to a question. It is processed very similarly, it just uses the same scope as its parent question
pub struct Answer {
    ///This is a list of expressions that need to be evaluated using the same variable values as its parent question
    pub expressions: Vec<Expression>,
    ///This is a list of the content in the Answer that doesn't need to be evaluated
    pub layout: Vec<String>
}

struct Content {
    vars: HashSet<Var>,
    expressions: Vec<Expression>,
    layout: Vec<String>
}

///An Expression represents a mathematical expression to be evaluated
pub struct Expression {
    ///This is a list of variables/other content that makes up the expression
    pub expression: Vec<ExpComp>
}

#[derive(PartialEq, Eq, Hash)]
///A Var holds information about a variable that is used to generate final values
pub struct Var {
    ///The variable name
    pub name: String,
    ///The type of the variable: either int or real
    pub num_type: String,
    ///The minimum value for this variable
    pub min: String,
    ///The maximum value for this variable
    pub max: String
}

///This is an enum used to differentiate between variable names and other content of an expression
pub enum ExpComp {
    ///This denotes a variable name
    Var(String),
    ///This denotes everything other than variable names
    Other(String)
}

#[derive(PartialEq, Eq, Hash)]
struct Num {
    whole: i64,
    frac: Option<i64>
}





///This function takes an input &str in the desired template format and generates a document. If the document has answers you should use process_with_answers.
///
/// # Arguments
///
/// * `input` - A string slice that holds the template contents for the Document
///
/// # Examples
///
/// ```
/// use morphius;
/// let doc = morphius::process("Document Contents");
/// ```
pub fn process(input: &str) -> Document {
    lazy_static! {
        static ref QUESTION: Regex = Regex::new(r"(?s)\|<q>(.*?)</q>\|").unwrap();
    }
    let questions: Vec<Question> = QUESTION.captures_iter(input).map(|cap| process_question(&cap[1], None)).collect();
    let layout: Vec<String> = QUESTION.split(input).map(String::from).collect();
    Document{ questions, layout }
}

///This function takes an input &str in the desired template format and generates a document. The input document must have answers provided for each question.
///
/// # Arguments
///
/// * `input` - A string slice that holds the template contents for the Document
///
/// # Examples
///
/// ```
/// use morphius;
/// let doc = morphius::process_with_answers("Document Contents with answers");
/// ```
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

///This function takes an input Document, the number of tests that you want to generate and optionally the number of question per generated test
///
/// # Arguments
///
/// * `doc` - A reference to a Document for the template that you want to generate
/// * `num_results` - The number of tests to generate
/// * `num_quesitions` - The number of questions per test. Enter None to use all questions in the original order. To include all questions and reorder them, enter `Some(x}` where x is the total number of questions
///
/// # Examples
///
/// ```
/// use morphius;
/// let doc = morphius::process("|<q>Example Question 1</q>||<q>Example Question 2</q>|");
/// morphius::generate(&doc, 5, Some(2));
/// ```
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
    let mut scope:HashMap<&str,Num> = HashMap::new();
    for var in question.vars.iter() {
        if var.num_type == "int" {
            scope.insert(&var.name[..], Num{ whole: rng.gen_range(var.min.parse::<i64>().unwrap()..(var.max.parse::<i64>().unwrap()+1)), frac: None});
        } else {
            let whole = rng.gen_range(var.min.parse::<i64>().unwrap()..var.max.parse::<i64>().unwrap());
            let frac: i64 = rng.gen_range(0..1000);
            scope.insert(&var.name[..], Num{ whole, frac: Some(frac) });
        }
    }


    let content = question.layout.iter().interleave(&question.expressions.iter().map(|exp| gen_expression_text(exp, &scope)).collect::<Vec<String>>()).join("");

    let answer: String = match &question.answer {
        Some(answer) => answer.layout.iter().interleave(&answer.expressions.iter().map(|exp| gen_expression_text(exp, &scope)).collect::<Vec<String>>()).join(""),
        None => String::from("No Answers Provided")
    };

    (content, answer)
}

fn gen_expression_text(expression: &Expression, scope: &HashMap<&str,Num>) -> String {
    let expr = expression.expression.iter().map(|exp_cmp| {
        match exp_cmp {
            ExpComp::Var(var_name) => {
                let num = scope.get(&var_name[..]).unwrap();
                match &num.frac {
                    None => num.whole.to_string(),
                    Some(frac) => (num.whole as f64 + (*frac as f64 / 1000f64)).to_string()
                }
            }
            ExpComp::Other(text) => text.clone()
        }
    })
    .join("");
    match mexprp::eval::<f64>(&expr).unwrap() {
        mexprp::Answer::Single(num) => {
            let rounded = format!("{:.3}", num);
            let normal = num.to_string();
            if normal.chars().count() > rounded.chars().count()  {
                rounded
            } else {
                normal
            }
        }
        mexprp::Answer::Multiple(_) => panic!("Unsupported math")
    }
}

fn process_question(question: &str, answer: Option<Answer>) -> Question {
    lazy_static! {
        static ref VAR: Regex = Regex::new(r"\|<v>([[:alpha:]][[:word:]]*):\s*([[:alpha:]]*)\s*=\s*\[(-?[0-9]+),(-?[0-9]+)\]</v>\|").unwrap();
    }
    let mut content = get_content(&VAR.split(question).join(""));
    for cap in VAR.captures_iter(question) {
        content.vars.remove(&Var{ name: String::from(&cap[1]), num_type: String::from("int"), min: String::from("0"), max: String::from("99") });
        content.vars.insert(Var{ name: String::from(&cap[1]), num_type: String::from(&cap[2]), min: String::from(&cap[3]), max: String::from(&cap[4])});
    }
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
    let mut vars: HashSet<Var> = HashSet::new();
    let expressions: Vec<Expression> = EXP.captures_iter(text).map(|cap| process_expression(&cap[1], &mut vars)).collect();
    let layout: Vec<String> = EXP.split(text).map(String::from).collect();
    Content{ vars, expressions, layout }
}

fn process_expression(expression: &str, vars: &mut HashSet<Var>) -> Expression {
    lazy_static! {
        static ref VAR: Regex = Regex::new(r"[[:alpha:]][[:word:]]*").unwrap();
    }
    let mut vars_list: Vec<ExpComp> = Vec::new();
    for cap in VAR.captures_iter(expression) {
        vars.insert(Var{ name: String::from(&cap[0]), num_type: String::from("int"), min: String::from("0"), max: String::from("99") });
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
        println!("{}", result[0].content);
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

    #[test]
    fn test_var_bounds_are_processed() {
        let doc = process("|<q>|<v>x: real = [5,55]</v>||<e>x/x</e>|</q>|");
        for result in generate(&doc, 3, Some(1)) {
            assert!(result.content == "1");
        }
    }

    #[test]
    fn test_numerical_rounding_to_three_decimal_places() {
        let doc = process("|<q>|<e>1/3</e>|</q>|");
        for result in generate(&doc, 3, Some(1)) {
            assert_eq!("0.333", result.content);
        }
    }

}