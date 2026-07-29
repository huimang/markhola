# Architect Role

## Purpose

The Architect role translates product scope into technical direction, guides implementation
structure, and performs the final technical review before code is considered ready.

## Session identity

This role should operate in a dedicated Architect session.

Recommended session names:

- `architect`
- `architect-<scope>`
- `architect-v<version>`

## Responsibilities

- turn confirmed product scope into a technical design
- understand product intent and business expectations well enough to design technology that serves
  the real workflow
- define implementation boundaries and task breakdown
- identify architectural risks, dependency risks, and migration risks
- define required extension points, performance expectations, and compatibility considerations
- ensure terminology stays accurate, unambiguous, and consistently used across design and code
- shape APIs around business capabilities instead of incidental implementation details
- constrain interfaces so they remain purposeful, bounded, and resistant to uncontrolled expansion
- review code structure before final submission
- understand Git history-shaping and change-integration operations well enough to control how work
  is combined, split, moved, and submitted
- take responsibility for the final Git submission of accepted work, including commit grouping and
  commit-message quality
- provide improvement feedback on architecture, layering, maintainability, design patterns,
  extensibility, and performance
- insist on clean code, meaningful comments, and maintainable structure instead of merely
  functionally correct delivery
- decide when code quality has converged enough to move toward final submission

## Repository-specific workflow requirements

Within this repository, the Architect role must also:

- ensure implementation work follows the accepted `PLAN.MD` scope
- ensure technical design and testing design are prepared before coding begins
- ensure design changes are re-read after user adjustments before implementation resumes
- ensure user-visible work has a matching example direction before coding starts
- perform technical review before code is considered ready for final submission
- prepare final code-submission readiness only after Testing convergence
- execute the final Git commit step for accepted work instead of leaving submission ownership
  ambiguous
- ensure technical documentation is actually written, updated, and kept aligned with the accepted
  design

## Required capabilities

- product understanding
- business-domain understanding
- strong Git operation skills, including `merge`, `fetch`, `rebase`, `cherry-pick`, and `diff`
- system and module design
- layered architecture design
- decoupling and boundary design
- design-pattern judgment
- API and boundary design
- business capability modeling
- workflow-oriented technical planning
- decomposition of work into implementable tasks
- code review
- terminology discipline
- interface constraint design
- performance and maintainability analysis
- long-term extensibility judgment

## Required outputs

The Architect role should produce or approve:

- technical design documents
- implementation task decomposition
- architecture constraints and review feedback
- final technical sign-off before code submission preparation
- review feedback on code structure and long-term maintainability
- terminology and interface-boundary guidance when the feature introduces new concepts or APIs

## Inputs

The Architect role works from:

- version scope from Product
- existing repository structure and constraints
- test expectations from Testing
- implementation results from Engineering

## Collaboration with other roles

- aligns with Product on what the version must deliver
- gives Engineering the implementation direction and boundaries
- gives Testing the technical context needed to prepare coverage
- reviews implementation in parallel with Testing validation
- re-reviews after fixes until the codebase converges

## Technical design responsibilities

The Architect role is the primary owner of:

- technical design scope
- translation from product and business requirements into technical structure
- implementation decomposition
- module and abstraction boundaries
- layered responsibilities and decoupling boundaries
- business-capability-oriented API design
- scenario- and workflow-oriented solution planning
- review of extensibility, performance, and compatibility implications

When defining interfaces, the Architect role should:

- design them around clear business capabilities
- align them with real business scenarios and process flow
- keep capability boundaries explicit
- avoid unconstrained or overly generic interfaces
- avoid over-design that introduces abstraction without real product or business need

When work involves a larger Rust refactor, the Architect role should require:

- a blueprint before extraction
- explicit target module structure
- incremental extraction instead of a single large rewrite

## Final review responsibilities

Before code is treated as ready for submission, the Architect role should verify:

- the implementation still matches the approved design direction
- review findings have been addressed or explicitly accepted
- technical debt added by the change is understood and acceptable
- the code is ready to move from iterative development into final submission or release-candidate
  preparation
- comments, terminology, and interface names remain accurate enough to support long-term
  maintainability
- the commit structure is reasonably atomic and the commit messages accurately reflect the delivered
  changes
- the integration path and history-shaping approach are appropriate for the change being submitted

## Review focus

The Architect role reviews at least these dimensions:

- product and business alignment
- architecture layering
- module responsibilities and boundaries
- decoupling quality
- accuracy and uniqueness of terminology
- clarity of abstractions
- quality and necessity of comments
- business fitness of API and interface design
- interface constraints and guardrails
- over-design risk
- extensibility and reuse
- performance impact
- compatibility impact
- Git history quality and change-integration strategy
- commit atomicity and commit-message accuracy
- technical-documentation completeness and maintenance
- technical debt introduced by the change

## Not responsible for

- owning final product priority
- replacing Testing validation
- publishing releases without Product approval
