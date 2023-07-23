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
- create / destroy entity
- add / remove component
- add / remove singleton
- mutable API call

### Entity and ownership

An entity is a combinaison of index and a version. The index is unique within a scene.

## System Execution Model

### System

A system is the smallest execution process. 

A system has a execution mode :
- Exclusive : it is guarranted to be executed alone. Therefore, it can apply structural changes. 
- Parallel  : executed in parallel, can't apply structural changes. Views can be used to make parallelism safe across the same type of component.

Systems are described by one of the module :
- Source Script     : compiled / interpreted source language.
- Node Script       : graph based script.
- Function Callback : rust function callback.

Execution mode can be inferred for source scripts or node scripts by scanning generated instructions. Function callback cannot be inferred and must be specified with Exclusive or Parallel API.

### SystemGraph

System execution order is specified by a system graph. A system graph describe system pipelines attached to an signal.
A system can be attached to a group and toggled at runtime. A system graph is composed of multiple steps. A step can be :
- Exclusive Call : call a single exlusive or parallel system.
- Parallel Call : call a list of parallel system (the engine is responsible for parallelism).
- Propagate : propagate the signal to a sub system graph.

Each step produce component changes or structural changes. Therefore, a flushing step can be apply at the end of the step. Flushing trigger reactive systems and apply changes.

### Signals

A pipeline is attached to a procedure. A procedure is invoked 
- Immediate : called after the current procedure.
- EndFrame  : called at the end of the frame (in order call).
- NextFrame : called at the beginning of the next frame (in order call).

### SystemGroup

A pipeline belongs to a system group and is attached to a procedure. The system group can
be enabled or disabled to control pipeline activation (enabled by default).

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