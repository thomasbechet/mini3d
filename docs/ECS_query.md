a -> all entity with a
+a -> all entity with added a since last system update (require keep track of last update)
-a -> all entity with removed a since last system update (require keep track of last update)
~a -> all entity with a changed since the last system update (use generation number)
!a -> all entity without a
a&b -> all entity with a and b
a|b -> all entity with a or b

a&(b|c) 

new().all()

+transform & +model
added component
removed component
enter group
leave group

set operators: & | !
filter operators: + - ~

component ::= identifier
filtered  ::= (+,-,~)component
group     ::= component (&,|,!) group
filtered_group ::= (+,-,~)group


resolver.query().any()

Matcher::new().all(transform, model).added(transform)
Matcher::n