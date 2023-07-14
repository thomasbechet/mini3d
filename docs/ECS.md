## Structural change

Any structural change prevent parallelism.
A structural change occurs on the following actions :
- create / destroy entity
- add / remove component
- add / remove singleton

## Buffered commands

Buffered commands are executed at the end of the procedure. Adding component or creating entity doesn't break iterators or query.

The following actions are delayed :
- destroy entity
- remove component
- remove singleton

## Scene

## Container

## Systems

Systems are 