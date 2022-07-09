# morphius

`morphius` allows users to randomize the order and content of documents
which can be used by teachers for generating tests with questions in a 
different order or with different numbers in each question for each student. If answers
are provided, morphius will generate an answer key for each final test to
make grading easier.

# Format
`morphius` processes content in a specific format described below:

##### Questions
A question in `morphius` is indicated in the following format:

`"|<q>Question Content</q>|"` The content inside the `"|<q>"` and the `"</q>|"` is
the question content. The questions 
will be rearranged if desired to create a different order of questions on different 
tests while the rest of the content in the template remains in the same place. This means
that if question numbers are included in the template, those should be placed outside
of the question itself in order to retain the correct numbering when questions are 
rearranged.

##### Expressions
Expressions are used to add randomness to questions:

`|<e>a+b</e>|` is an example expression. Expressions must be placed inside of a question
in order to be processed. Expressions are a mathematical expression that can include variables and 
evaluates to a number when generating a test. Variables represent a random number that will be selected
separately for each test generated. A variable in an expression is any identifier that starts with 
a letter followed by a sequence of characters that can contain letters, numbers or underscores.
Variables do not have to be declared and will default to be an integer between 0 and 99 when generated.
Math is allowed in these expressions in order to create specific relationships between numbers in the generated
question. Math is supported using the `mexprp` crate. The scope of variables is the question, so you can have
the same variable names in different questions and they will likely have different values (unless they randomly
end up being the same). If you want more fine tuned control of the range of possible values, you can declare
the variable.

##### Variable Declarations
A variable can be declared anywhere in the question in the following format:

`|<v>var_name: type = [min,max]</v>|` where var_name is the name of your variable, type is either int or real, and min and max
are integers representing the lower and upper bounds respectively of the value of your variable. An example declaration would be
`|<v>a: int = [0,99]</v>|`. This is the declaration assumed for any variable without a declaration, so including this exact
declaration in your code would be unecessary.

##### Answers

Answers are used to generate an answer key for each test. Answers should be included for every question when using
`process_with_answers`. They should be in the format `|<a>Answer</a>|` and should appear right after the question. Variables
in answers have the same scope as their corresponding question so you can use expressions in your answers to calculate the
answer in terms of the randomly generated variables in your question.


# Examples
Here is a simple example, you can find more example templates in the `examples` folder of the GitHub repository.
```
let template = "
|<q>This is a single question test. You must calculate the sum of two random numbers.
What is |<e>a</e>| + |<e>b</e>|?</q>|
|<a>The answer to this question is: |<e>a+b</e>|</a>|";

let doc = morphius::process_with_answers(template);
let tests = morphius::generate(&doc, 5, Some(1));

//Prints out the first test
println!("{}", tests[0].content);

//Prints out the answer key for the first test
println!("{}", tests[0].answers);

```

