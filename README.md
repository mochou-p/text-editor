<!-- text-editor/README.md -->

# text editor
simple text-based text editor

# configuration
file location is in the root of this project `./config.conf`  
you can ommit settings, and default values will be used  
when no file is present, a default one like this will be generated:
```py
alignment-horizontal = left    # left/center-left/center/center-right/right
alignment-vertical   = center  # top/center/bottom
```

# configuration properties
### alignment-horizontal
```
left         center       right
+---------+  +---------+  +---------+
|#####    |  |  #####  |  |    #####|
|###      |  |   ###   |  |      ###|
|#        |  |    #    |  |        #|
|###      |  |   ###   |  |      ###|
+---------+  +---------+  +---------+

center-left  center-right
+---------+  +---------+
|  #####  |  |  #####  |
|  ###    |  |    ###  |
|  #      |  |      #  |
|  ###    |  |    ###  |
+---------+  +---------+
```
### alignment-vertical
```
top          center       bottom
+---------+  +---------+  +---------+
|#########|  |         |  |         |
|         |  |#########|  |         |
|         |  |         |  |#########|
+---------+  +---------+  +---------+
```

## License
Licensed under either of
 * Apache License, Version 2.0  
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license  
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions

