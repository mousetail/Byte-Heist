# How to Write a Challenge

Creating new challenges and watching other people come up with crazy unexpected solutions is one of the most fun parts of code golf. 

## Comping up with the idea

Some good places to get ideas include:

- The [Online Encyclopedia of Integer Sequences](https://oeis.org/)
- The [Wikipedia list of Algorithms](https://en.wikipedia.org/wiki/List_of_algorithms)
- The list of Beta challenges on other coding challenge sites, particularly ones that are stalled because of the sites limited judging flexibility but would work in Byte Heist's more flexible one

Ideas are cheap.

## Validating and refining your idea

After you have found an idea, you need to check if it would actually work. Some things to keep in mind:

- Basic, but important to ask: Is it actually possible to solve? Consider the various sand-boxing restrictions present on Byte Heist as well as the time limit. Ideally the challenge is theoretically solvable in every language.
- Is solving it "properly" likely shorter than just hard coding or hashing? It is not fun to put a lot of effort into solving a challenge only for people to get a better score by just using compression or hashing the inputs.
- Is it sufficiently different from existing challenges? Solving challenges is most fun when you need to figure out new algorithms, not just shift your existing solutions around a bit. Challenges that look very different can still actually have the same optimal solutions as each other.
    - A challenge can also be boring if it is just multiple common challenges stitched together. Even if the combination is new, that does not make the challenge good.
    - Consider duplicates on other sites too. It can be OK to create challenges similar to ones on other sites if Byte Heist's flexible judging would improve them or if they are "staples of the genre". However, exact duplicates are not nice.

Asking on the Discord is also a good way to test the viability your challenge.

## Creating a Judge Function

If you think there is a real good challenge left from your idea, it's time to create a judge function.

You can create a heist at [byte-heist.com/challenge](https://byte-heist.com/challenge). 

Judges are currently written in Deno. The library of functions available to you are here: [runner-lib.ts](https://github.com/mousetail/Byte-Heist/blob/master/lang-runner/scripts/runner-lib.ts). The exact process changes too much so this documentation would be instantly out of date. Existing challenges should provide a good base.

Byte Heist's judging system is quite flexible, ideally the judge should be programmed to ignore noise not directly related to the algorithm, like order of outputs or spacing.

### Tips for preventing 🧀

"Cheese" or "🧀" refers to solutions that should not actually solve the challenge but pass because of luck or a weakness in the judge or testing infrastructure. Since we
can only use a finite amount of edge cases, some 🧀 is unavoidable, but the best solutions should hopefully be mostly legitimate and only save a handful of bytes due to 🧀.

Before we can prevent 🧀, we need to explain what techniques can be used to 🧀:

- If there are sometimes no test cases that test a specific edge case, golfers can assume this will never happen and simplify branches or flip conditionals, then just run the solution until no edge case test case is generated
    - The fix for this is hard coding edge cases to guarantee they will appear in every test
- **Hashing Test Cases** refers to a technique where you base some decision on an ephemeral property of a test case. For example, if the only test case to exploit a certain edge case happens to have a unique `sum%7`, it could be shorter to branch of the `sum%7` than actually checking for the proper condition.
    - The fix for this is to have randomly generated test cases, ideally including randomly generated ones around edge cases.
- Assuming test cases exists. Sometimes it's possible to 🧀 based on other test cases in a run. This includes techniques like taking a string from the input, or hashing based on the sorting order of a test case. For example, if a test case that tests an edge case is `aaaaabb`, it could be shorter to check if the current test case is alphabetically first than properly testing the property.
    - The fix is to randomly distribute the test cases over multiple runs. If you use `Context.runTestCases` or `Context.runFilterCases`, this is done for you automatically.

Increasing the number of test cases and/or runs is a very effective way to cut down on 🧀, but be mindful of timeouts. A naive solution on a slow language should ideally still be able to solve the challenge.

🧀 prevention is very much a community effort, watch the discord to find people talking about 🧀. Anyone can patch 🧀 when it's found, not just the challenge author.

Sometimes, rather than "patch" a 🧀 it's better to instead "bless" by officially adding an assumption to the challenge description. Exploiting a specific assumption is fun, and as long as it properly documented it is fair game.

## Writing a challenge description

Writing good descriptions for challenges is hard, but very important. After all, people read the description before deciding to even solve it or encountering the judge.

The most important bit when writing challenges is to be precise. Describe the input and output format, describe what assumptions you can and cannot make on the input. Explain limits and ranges.

This is also the time to check if your challenge is really as complex as you thought. Quite often, there is a much simpler challenge that is actually equivalent with the assumptions given. It can be good to actually describe multiple ways to view the same problem. For example, if your challenge was the [Catalan Numbers](https://en.wikipedia.org/wiki/Catalan_number) you could give both the formula `factorial(2*n)/factorial(n+1)/factorial(n)` and say it's the number of ways to create balanced parenthesis in a string of some length and explain how it's the number of non-crossing partitions of a n-set.

You can include a lot of external links. Space is limited so you cannot include every mathematical fact so link there. Link to Wikipedia, OEIS, RosettaCode, ArXiv and any other resource you can find that would give more background on the problem.

Some flavor text near the top explaining background can be OK, especially for easier challenges, But keep it short.

## Publishing your challenge

If you are done writing your challenge, it's time to publish a beta. The beta is when people can attempt your challenge but get no score. This is when people can provide feedback and suggest changes.

We pick a challenge from the beta pool to go live around once a month, and usually try to alternate between code golf and restricted source.