# Registries

Registries are shared with all activities.

# Resources

Resources represents data that can be shared across multiple activities. A resource is released when its reference count drops to zero. An activity doesn't own an activity. However, it can reference an anonymous resource (i.e. procedural mesh) which can't be directly accessed by other activities.

# Activities

An activity can be considered as an ECS.