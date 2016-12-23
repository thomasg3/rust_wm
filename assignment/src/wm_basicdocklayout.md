# Basic Dock Layout #

 After adding the first tile

 ```
 +---------------------+
 |                     |
 |                     |
 |          1          |
 |                     |
 |                     |
 +---------------------+
 ```

 Adding a second tile creates a left dock which takes 1/5th of the screen width

 ```
 +---------------------+
 |   |                 |
 |   |                 |
 | 2 |      1          |
 |   |                 |
 |   |                 |
 +---------------------+
 ```

 Adding a third tile creates a right dock which takes also 1/5th of the screen width

 ```
 +---------------------+
 |   |             |   |
 |   |             |   |
 | 2 |      1      | 3 |
 |   |             |   |
 |   |             |   |
 +---------------------+
 ```

 Adding a fourth tile creates a bottom dock which takes the remaining width, and 1/5th of the
 height of the screen.

 ```
 +---------------------+
 |   |             |   |
 |   |      1      |   |
 | 2 |             | 3 |
 |   |-------------|   |
 |   |      4      |   |
 +---------------------+
 ```

 Adding additional tiles will divide the following dock (in the same order) in equal parts
 Removing a tile, will however move over the entire structure, for example removing the second
 tile in the fourth example will render this

 ```
 +---------------------+
 |   |             |   |
 |   |             |   |
 | 3 |      1      | 4 |
 |   |             |   |
 |   |             |   |
 +---------------------+
 ```

 One is the master tile. Swapping with the master tile works as expected, as does swap windows
