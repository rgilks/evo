Group related components near each other
Organize code by domain, not technical concerns

## Rayon Parallelism
Use Rayon for CPU-intensive operations on large datasets
Prefer par_iter() over manual thread spawning
Use par_iter_mut() for mutable operations

## Hecs ECS Design
Keep components small and focused on one aspect
Query only what you need with specific component combinations
Use Query for read-only access, QueryBorrow for mutable access
Group related systems together

Avoid log flooding with appropriate log levels
Use rustfmt for consistent formatting
Use clippy and fix all warnings
Write idiomatic Rust patterns
Separate evolution logic from rendering/UI
Design for large populations and scalability
Make simulation parameters easily adjustable
Enable easy experimentation and modification
Simplicity first - code should be concise and elegant
Keep README file up to date
Keep .gitignore file up to date
Be rigorous in fixing warnings and errors
Code files should preferably be less than 200 lines long.
Functions should preferably be less than 20 lines long.
