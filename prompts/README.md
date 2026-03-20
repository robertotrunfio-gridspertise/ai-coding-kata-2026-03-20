# Prompts Folder

Store in this folder the prompts that were used during code generation, analysis, refactoring, and test creation for this kata.

Each prompt file should document the actual prompt text that was used, so the work can be reviewed and the prompting process can be reproduced.

# Prompt 1
we have to work on this repository to change the current code maintaining it's behaviour, but symplifying it. We will work only on the rust code, keepng go and java code out of our scope. The first thing i want to ask you is to summarize what is in the lib file, to understand exaclty what have already been done and why it is hard to undertand, maintain and extend

## Purpose
Understand the current state of the code and identify areas that are hard to understand, maintain, and extend.

## Outcome
A clear summary of the current implementation, highlighting the issues that need to be addressed in the ref

# Prompt 2
Ok, i have understood the situation. What we have to do now is to refactor all the code in order to simplify it.
The important thing is to MAINTAIN the behaviour. Everything must work as it does now, but with different implementation.
Let's change the implementation to make it extendable and maintainable. Rewrite the if/else statement, adjust the stringly-typed domain and remove the magic numbers. Add separate function for better code understanding and management.

## Purpose
Refactor the code to simplify it while maintaining its current behavior. The goal is to make the codebase more extendable and maintainable by improving the structure and readability.

## Outcome
A refactored version of the code that preserves existing functionality but is easier to understand, maintain, and extend in the future. The new implementation should eliminate the use of magic numbers, reduce stringly-typed domain, and improve the overall structure of the code.

# Prompt 3 
Nice! Now we have to add a new customer type. It have to be called "partner".
The partner has base discount of 12%
free shipping when discounted subtotal is at least 15000 cents
coupon "PARTNER5" adds an extra 5% discount only for partner customers and only when subtotal is at least 12000 cents
on Black Friday, partner customers get an extra 3% discount instead of the usual 5%

add more tests for this new customer type

## Purpose
Add support for a new customer type called "partner" with specific rules for discounts and shipping. The implementation should maintain the existing behavior for other customer types while integrating the new rules for partners. Additionally, new tests should be created to ensure that the functionality for the partner customer type is correctly implemented and does not affect existing functionality.

## Outcome
The codebase will be updated to include the new "partner" customer type with its associated rules for discounts and shipping. The implementation will ensure that the existing behavior for other customer types remains unchanged. New tests will be added to validate the correct behavior of the partner customer type, including scenarios for base discount, free shipping, coupon usage, and Black Friday discounts. The overall code will remain maintainable and extendable, allowing for future additions without significant refactoring.

# Prompt 4
let's create a new markdown file in the same directory. The file must be called "solution.md" and will contain the following information:

- what was wrong in the legacy design
- what changed
- why the new structure is easier to extend

## Purpose
Document the changes made to the codebase, including the issues with the legacy design, the specific changes implemented, and the reasons why the new structure is easier to extend.

## Outcome
A comprehensive markdown file named "solution.md" that provides a clear explanation of the problems with the legacy design, the changes that were made to address those problems, and the benefits of the new structure in terms of extensibility. This documentation will serve as a reference for future developers and help them understand the rationale behind the refactoring decisions and how to work with the new codebase effectively.