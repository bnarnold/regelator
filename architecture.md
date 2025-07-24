# Regelator - Architecture & Design Document

## Overview

Regelator is a backend-first application for managing rules for the sport of Ultimate Frisbee.
It provides easily accessible interfaces for learning about the rules and using them in games.

## Target Users

There are different kinds of users, exemplified by the following personas:

### Nathan Newman

Nathan recently picked up Ultimate Frisbee and got hooked.
He wants to learn more about the rules, but reading them cover to cover is too dry for him.
Gamification helps him stay interested in the rules and discover new aspects that did not come up in his group yet.
He also wants a reference that he can ask free-form questions when they come up in games.

### Valeria Veterana

Valeria has been playing Ultimate Frisbee for a long time.
She has taken the advance rules knowledge certification and knows not just the core concepts, but also various corner cases.
She wants to stay up to date with rules changes and have a quick reference to share with teammates.

### Fred Federator

Fred is in the rules committee of his national federation and coordinates rule translations.
He wants to enable members of his community to learn about the rules in their native language.
He supports translators by providing tools to compare translated versions with the English original.
To target the committee's actions, he also wants to collect statistics about application usage and identify commonly referenced rules.

## Frontend Architecture

The frontend uses server-side rendering with MiniJinja templates and htmx for interactivity.

### Template Structure
- `base.html` - Main layout template
- Page-specific templates extend the base template

### HTML/CSS Principles
- **Semantic HTML First**: Always use semantic HTML elements (`<ol>`, `<li>`, `<section>`, etc.) for proper structure and accessibility
- **CSS for Presentation**: Use CSS to control visual presentation, not HTML structure (e.g., `list-style: none` to hide browser numbering while keeping semantic `<ol>`)
- **Progressive Enhancement**: Base functionality works without JavaScript, enhanced with htmx

### Rule Display Strategy
- Rule hierarchies use semantic `<ol>` and `<li>` elements
- Browser-generated numbering hidden via CSS (`list-style: none`)
- Rule numbers are clickable links for direct navigation
- Actual rule content displayed instead of generated titles
