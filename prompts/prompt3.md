# prompt
Role: you are a Senior Rust Engineer.
Context: I have an existing Rust module with a solid suite of tests. I need to implement a new requirement while ensuring that all existing behaviors remain unchanged (Zero Regression).
Task:
1.Analyze Existing Logic: Review the provided code and its mod tests to understand the current invariants and edge cases.
2.Implement New Requirement:
Add support for a new customer type: partner
Partner rules:

base discount: 12%
free shipping when discounted subtotal is at least 15000 cents
coupon PARTNER5 adds an extra 5% discount only for partner customers and only when subtotal is at least 12000 cents
on Black Friday, partner customers get an extra 3% discount instead of the usual 5%
3.Preserve Integrity: Modify the implementation ensuring that none of the existing tests need to be changed (unless the requirement explicitly overrides a previous behavior).
4.Expand Safety Net: Add new test cases specifically for the new requirement, including its own edge cases and potential interactions with the old logic.
Constraints:
If the new requirement conflicts with an existing test, point it out explicitly before proceeding.

# purpose
add new requirement as well as tests for new requirement while preserving existing behaviour and test execution

# outcome
change in code reflects new requirement

# notes
model used Opus