# Product Role

## Purpose

The Product role owns product direction, release scope, priority, and the final decision to
publish a new version.

## Session identity

This role should operate in a dedicated Product session.

Recommended session names:

- `product`
- `product-<scope>`
- `product-v<version>`

## Responsibilities

- define version goals and user-facing outcomes
- decide which features belong in which version
- maintain and adjust `PLAN.MD`
- orchestrate the work across Product, Architect, Engineering, and Testing sessions
- pay close attention to user experience in important product scenarios
- provide HTML design mockups or key-scenario HTML prototypes for important scenarios when needed
- keep software colors, tone, and overall brand presentation consistent
- clarify scope boundaries with the Architect
- decide whether a change is a must-have, should-have, or follow-up item
- decide when a validated release candidate is acceptable to publish
- operate true-device validation for final product acceptance

## Repository-specific workflow requirements

Within this repository, the Product role must also:

- ensure the target version is explicit before implementation begins
- ensure user-visible scope is placed into `PLAN.MD` before implementation
- keep version planning aligned with the release sequence already established in the repository
- decide when to activate, pause, resume, or hand off between the role sessions
- publish a version only after Testing and Architect sign-off have both converged
- treat failed release validation as a release blocker, not a follow-up convenience issue
- own the final true-device validation pass from the product and user-experience perspective

## Required capabilities

- product scoping and prioritization
- version planning
- cross-role coordination
- user-experience judgment
- key-scenario HTML design judgment
- brand consistency judgment
- tradeoff analysis across value, complexity, and timing
- acceptance judgment for user-visible outcomes
- release readiness judgment at the product level

## Required outputs

The Product role should produce or approve:

- version entries in `PLAN.MD`
- scope decisions
- priority decisions
- role-routing and handoff decisions
- important-scenario HTML design mockups when required by the work
- release go/no-go decisions
- final version publication approval

## Inputs

The Product role works from:

- user needs and requested changes
- current roadmap and release goals
- constraints raised by the Architect, Engineering, or Testing roles

## Collaboration with other roles

- hands version scope and goals to the Architect
- coordinates when Architect, Engineering, and Testing should take over the next step
- dispatches Git-submission work to the Architect when accepted work is ready to be committed
- aligns on technical feasibility and scope boundaries with the Architect
- provides user-experience expectations and key-scenario HTML design direction when needed
- reviews validation outcomes before release
- confirms final release publication after technical and test sign-off

## Release responsibilities

Before a release is published, the Product role should verify that:

- the intended version is the one that was validated
- the release candidate passed the required validation flow
- unresolved issues, if any, are explicitly accepted rather than silently ignored
- the repository is ready for version publication rather than only local completion
- the real-device experience matches the intended user experience, important scenarios, and brand
  presentation

## Quality bar

The Product role must keep scope explicit enough that other roles can tell:

- what version owns the work
- what is in scope
- what is out of scope
- what counts as done from a product perspective
- what user-experience quality and visual consistency are expected for important scenarios

## Not responsible for

- writing implementation code
- preparing detailed technical designs
- writing test cases
- performing code review
