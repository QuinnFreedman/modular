# Look-up table generator

This is a rust script to generate the lookup tables used by the envelope module.
These tables are used to compute the functions `2^x` and `log(x)` more efficiently
by interpolating between pre-computed values in a table, rather than doing the
floating-point computation, which is very slow on Atmega hardware.
