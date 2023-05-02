# Custom Shader Support

A `Renderable` trait object is cached in a map with an associated pipeline identifier key to prevent multiple 
instances of the same pipeline from being created.

The `Renderable` object is *not* reallocated every frame; it simply references a "state" struct which is defined by 
the user. This means that the state of any primitives must be managed **by the user**.

Users can indicate that a custom shader exists that needs to be rendered in a certain order by "slotting" it in to 
the existing list of primitives with a new variant of `Primitive` called `Custom`. A `Custom` primitive takes an 
initializer fn pointer which returns the `Renderable` trait object, as well as contains the unique identifier of the 
pipeline. When a user indicates that they wish to render this custom shader in a widget, the 
unique pipeline id is added to the current layer. That ID is then used for a lookup to perform the `Renderable`'s 
`prepare` and `render` methods during the render pass.

**Pros:**
- Flexible
- No boxing of primitive data + no reflection needed

**Cons:**
- State must be managed by the user and isn't inherently tied to the widget
- In general feels a bit hacky
- Dynamic dispatch of every draw call
- Not flexible enough for all use cases
  - For example, just needing to add a simple depth stencil texture to the `cubes` example required significant 
    changes in the interface.