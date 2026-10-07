# Security Policy

## Reporting a vulnerability

Please do not open a public issue for a security problem. Use GitHub's private
[security advisory](https://github.com/gDarbvy2015/Palisade/security/advisories/new) form, or contact a
maintainer directly if the form is unavailable to you.

Include what an attacker gains in one sentence, the version or commit you are running, the smallest
reproduction you can manage, and whether the input is attacker-controlled in your setup. You should
get an acknowledgement within a few working days and a disclosure timeline before anything is
published.

## Scope

Palisade decides whether and how a command is launched from a declarative policy.
In scope: policy text that admits a command it should refuse (args/env/path wardens
bypassed by crafted quoting or spacing), a receipt that does not match what actually
ran, and path handling in policy resolution.
