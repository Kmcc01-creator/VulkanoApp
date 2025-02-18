@radekmie
About · Blog · GitHub · RSS · Timeline
On Reusing Arc and Rc in Rust

By Radosław Miernik · Published on 30 March 2024 · Comment on Reddit
Table of contents

    Intro
    Data structure
    First implementation
    Problem statement
    Second implementation
    Closing thoughts

Intro

I wrote my first non-trivial program in Rust back in late 2019. Since then, I rewrote most of my PhD’s code in Rust (it’s not public yet), ported the entire Legends of Code and Magic engine, and even published a somewhat popular changestream-to-redis. It still feels fresh, but I’d say I’m fluent in it.

The more I code in Rust, the more I look into optimizations. Partially because I finally feel confident with my code, partially because I have even more skin in the game (I’m getting paid to code in Rust now), and partially because I just love the feeling of making things go faster. (Even if it’s not needed.)

Recently, I wanted to optimize one of the many AST passes in my PhD’s code, annotating arbitrary expressions with types. The idea is fairly simple: analyze subexpressions recursively, infer the expression type, and add the annotation. But that involves a lot of cloning…
Data structure

The expressions are inherently recursive. As such, they will require some indirection to make Rust happy. Currently, they’re structured as follows:

enum Expression<Id> {
    // lhs[rhs]
    Access {
        lhs: Arc<Self>,
        rhs: Arc<Self>,
    },
    // lhs(rhs)
    Cast {
        lhs: Arc<Type<Id>>,
        rhs: Arc<Self>,
    },
    // identifier
    Reference {
        identifier: Id,
    },
}

There are at least a few alternatives to Arc, but…

    …Expressions are heavily reused, so Box would be a waste (.clone() is not as cheap as with Rc and Arc).
    …some operations are trivially parallelizable, so Expression needs to be Send (i.e., it can be sent to another thread); Rc is !Send, so it’s also out.
    …some operations create new Expressions, e.g., constant folding. While we are deep in the tree, we’d need to have some “external storage” for the original values so they could be borrowed. It’s doable, but the complexity would be significantly higher. So, borrows are also out.
        The same goes for using an arena allocator and replace nodes with their IDs. Maybe I’ll look into it in the future again, though.

…so Arc it is.
First implementation

Let’s try to implement the add_casts function. It should add a Cast on top of each Expression using the type it got from infer. It’d look like this:

// The below code is just a simplification -- it may not compile!
impl<Id> Expression<Id> {
    fn add_casts(&self, type_: &Type<Id>) -> Result<Self, Error<Id>> {
        let mut clone = match self {
            Self::Access { lhs, rhs } => {
                let lhs_type = lhs.infer()?;
                let Type::Arrow { lhs: key_type, .. } = lhs_type.resolve()? else {
                    return Err(ErrorReason::ArrowTypeExpected { got: lhs_type });
                };

                Self::Access {
                    lhs: lhs.add_casts(&lhs_type)?,
                    rhs: rhs.add_casts(key_type)?,
                }
            },
            Self::Cast { lhs, rhs } => rhs.add_casts(lhs)?,
            Self::Reference { .. } => {},
        };

        if !matches(clone, Self::Cast { lhs, .. } if *lhs != *type_) {
            clone = Self::Cast {
                lhs: Arc::new(type_),
                rhs: Arc::new(clone),
            };
        }

        Ok(clone)
    }
}

It’s a direct implementation of what we described, right? Note that it takes &self and returns Self: that means it always clones the entire Expression. But what if it’s not needed?
Problem statement

The more AST transformations there are, the more I have to think about the order they are applied in. Instead of spending hours experimenting what’d be the ideal one, I decided to do all of them in a fixed-point loop.

As a result, a lot of transformations are not doing anything (as expected). For example, if add_casts already added all Casts, then the next loop iteration will not have anything to do – that’s fine.

But as we already said, it’ll clone all of the subexpressions anyway. When profiled, roughly 30% of the time is spent in memcpy. It’s a lot, considering that all of them are thrown out immediately.

Maybe we could reuse these Arcs somehow?
Second implementation

It turns out we can, and it’s really simple: meet Arc::make_mut. It takes an Arc and returns a mutable reference to the inner value. Of course, it works only if it’s not reused (i.e., the strong count is 1), so it has to clone it if needed.

There’s a complexity cost, though: we have to have a &mut Arc, i.e., all of our transformations have to take either self or &mut self. Let’s see what the code could look like:

// The below code is just a simplification -- it may not compile!
impl<Id> Expression<Id> {
    fn add_casts(&mut self, type_: &Type<Id>) -> Result<(), Error<Id>> {
        match self {
            Self::Access { lhs, rhs } => {
                let lhs_type = lhs.infer()?;
                let Type::Arrow { lhs: key_type, .. } = lhs_type.resolve()? else {
                    return make_error(ErrorReason::ArrowTypeExpected { got: lhs_type });
                };

                Arc::make_mut(lhs).add_casts(&lhs_type)?;
                Arc::make_mut(rhs).add_casts(key_type)?;
            }
            Self::Cast { lhs, rhs } => {
                Arc::make_mut(rhs).add_casts(type_)?;
            }
            Self::Reference { .. } => {}
        }

        if !matches(self, Self::Cast { lhs, .. } if *lhs != *type_) {
            *self = Self::Cast {
                lhs: Arc::new(type_),
                rhs: Arc::new(self),
            };
        }

        Ok(())
    }
}

The code is not that different, but you can see where it’s going. Now, instead of constructing new Expressions at each step, we mutate them directly, even though they’re wrapped in Arcs. (Lazy cloning happens in the background.)
Closing thoughts

The second implementation allocates only when needed – that’s awesome! If we look at the numbers, it’s almost twice as fast. I believe the actual gains will differ based on how many (and how large) clones they were, but it seems like it’ll always help. At a cost of mutability, of course.

This was a short one, but maybe it’ll help some of you just diving into Rust.
On Getting Paid for Open Source
On Why Interfaces Matter