- Feature Name: lsdyna-lsp
- Start Date: 2023-03-10

# Summary
[summary]: #summary

A plan is proposed to explore the posibility to do simulation before
actually having a product design with main-stream CAE solvers.

CAE solvers normally come with a [scripting language], which is more
powerful than GUI yet lack of general usability for production.
There are [projects] tried to make CAE scripting easiar,
but to achieve the  goal of this [RFC],
we have to make CAE scripting easy, namely targeting not those who already do scripting,
but the GUI users who never does.

The coding part is comparably trivial, [language-server-protocal] and the [editors]
who support it would save us tons of code, and we are using [rust] to guarantee
software efficiency and reliability. The real kink would be how to properly tell the GUI story
in an IDE, this RFC is here to find a answer.

# Motivation
[motivation]: #motivation

We've been practising Simulation-Driven design(SDD) with manual force for pair of years,
SDD means you have to do simulation without an initial design, which doesn't work well in a GUI,
where you must have a design to apply the simulation on.
So what we were actually practising was, draw some naive design, apply the simulation,
draw a less naive desgin, apply the same simulation. We apply the same simulation in every
iteration and it feels stupid.

If I have to build a house on the river, at least let me build a boat first.

The boat should work as an IDE for a start for 2 reasons.

1. Every GUI operation is actually editing the script to submit to the solver,
editing the script directly unlocks all the potential.
2. People write codes with IDE support nowadays, they all do, why don't we?

On top of that, if we manage to leave every design related field into a variable,
and write everthing else as a function, then we have a black box where you feed it a design,
you get the simulation result.
We might have to develop a functional DSL to go that far,
but [nix-language] has set a great example for us how a DSL can achieve.

The first CAE solver we choose is ls-dyna,
it makes great animation, and we had the most experience,
I've made a toy application [duckbubble],
where you describes you simulation with a toml file,
and it automatically manages the relations of imported meshed-desgin with elments & materials.
The fun story is, when I introduce it to a new student in the group,
he get used to it quick and never had to learn how to do those steps in GUI.

I believe we could use more of that.


# Guide-level explanation
[guide-level-explanation]: #guide-level-explanation

Explain the proposal as if it was already included in the language and you were teaching it to another Rust programmer. That generally means:

- Introducing new named concepts.
- Explaining the feature largely in terms of examples.
- Explaining how Rust programmers should *think* about the feature, and how it should impact the way they use Rust. It should explain the impact as concretely as possible.
- If applicable, provide sample error messages, deprecation warnings, or migration guidance.
- If applicable, describe the differences between teaching this to existing Rust programmers and new Rust programmers.

For implementation-oriented RFCs (e.g. for compiler internals), this section should focus on how compiler contributors should think about the change, and give examples of its concrete impact. For policy RFCs, this section should provide an example-driven introduction to the policy, and explain its impact in concrete terms.

# Reference-level explanation
[reference-level-explanation]: #reference-level-explanation

This is the technical portion of the RFC. Explain the design in sufficient detail that:

- Its interaction with other features is clear.
- It is reasonably clear how the feature would be implemented.
- Corner cases are dissected by example.

The section should return to the examples given in the previous section, and explain more fully how the detailed proposal makes those examples work.

# Drawbacks
[drawbacks]: #drawbacks

Why should we *not* do this?

# Rationale and alternatives
[alternatives]: #alternatives

- Why is this design the best in the space of possible designs?
- What other designs have been considered and what is the rationale for not choosing them?
- What is the impact of not doing this?

# Unresolved questions
[unresolved]: #unresolved-questions

- What parts of the design do you expect to resolve through the RFC process before this gets merged?
- What parts of the design do you expect to resolve through the implementation of this feature before stabilization?
- What related issues do you consider out of scope for this RFC that could be addressed in the future independently of the solution that comes out of this RFC?
