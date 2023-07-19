## Buffered commands

Buffered commands are executed at the end of the procedure. Adding component or creating entity doesn't break iterators or query.

The following actions are delayed :
- destroy entity
- remove component
- remove singleton

## Scene

### Container

### Structural change

Any structural change prevent parallelism.
A structural change occurs on the following actions :
- creating / destroying entity
- adding / removing component
- adding / removing singleton
- transfering entity from / to another scene

### Entity and ownership

An entity is a combinaison of index and a version. The index is unique within a scene.

## System Execution Model

### System

A system is the smallest execution process. A system has a execution mode :
- Exclusive : it is guarranted to be executed alone. Therefore, it can apply structural changes. 
- Parallel  : executed in parallel, can't apply structural changes. Views can be used to make parallelism safe across the same type of component.

Systems are described by one of the module :
- Source Script     : compiled / interpreted source language.
- Node Script       : graph based script.
- Function Callback : rust function callback.

Execution mode can be inferred for scripts by scanning
instruction codes.

### Pipeline

A system belongs to a pipeline. A pipeline describe execution order of systems and possible
parallelism if systems. A system pipeline can be :
- Linear Pipeline : simple linear execution order (no parallelism).
- Graph Pipeline  : describe dependencies between systems and allow parallel execution.

A pipeline is attached to a procedure. A procedure is invoked 
- Immediate : called after the current procedure.
- EndFrame  : called at the end of the frame (in order call).
- NextFrame : called at the beginning of the next frame (in order call).

### SystemGroup

A pipeline belongs to a system group and is attached to a procedure. The system group can
be enabled or disabled to control pipeline activation (enabled by default).

Pipelines within the same procedure are executed in order based on their priority.

### Scheduler

The scheduler owns a list of system group and dispatch system calls. Only one scheduler exists
in the engine and belongs to the ECS manager.

### View Data Structure

- SceneComponentViewRef
    - Static : Ref<AnyStaticComponentVec>
    - Dynamic : Ref<AnyDynamicComponentVec>
    - None
- SceneComponentViewMut
    - Static : Mut<AnyStaticComponentVec>
    - Dynamic : Mut<AnyDynamicComponentVec>
    - None