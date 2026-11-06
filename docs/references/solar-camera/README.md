# Narrower solar-system camera

The overview now uses a 42Â° vertical field of view: [current capture](overview-42deg.png). [Previous 45Â° capture](overview-45deg.png).

Flight now uses 44Â° instead of 47Â°, with the chase camera farther back to keep the ship close to its previous on-screen size: [current capture](flight-44deg.png). [Previous 47Â° capture](flight-47deg.png).

The current overview shows tighter orbits (576 game units between orbit radii) and six starting positions distributed every 60Â° around the Sun. The backdrop now has a continuous blue and violet nebula field that rotates smoothly with the camera, plus colored stars and Advent Pro labels; orbit paths are visible again and can be toggled with O. The bottom-right AYUDA button opens the blue [controls frame](help-frame.png). Stars outside the field of view are culled before projection.

The entry starts with the [station surrounding the ship](entry-station.png), [tilts toward the solar system](entry-transition.png), and reaches [normal flight](entry-flight.png). The post-fade hold is 0.15 seconds and the camera move is 1.35 seconds. Controls become active in the last 0.15 seconds of the camera move. The ship starts on the station centerline.

The flight camera offset is shared with station departure clearance. The intro smoke and six-ship departure tests passed.
The launch now applies flight acceleration during the camera move and coasts at the handoff speed until the player uses the throttle. The extended intro smoke check verifies the ship is still moving after three seconds; see the updated [flight capture](entry-flight.png).
Floating planet names now use their universe logos in the [overview](overview-logos.png) and [flight](flight-logos.png); Halloween remains a text label. Flight distance numbers remain visible.
The floating logos are now smaller to keep the orbit view clear.
For the flat Smiling Friends planet, the label now follows the projected disk rim instead of a spherical radius. Compare the [close overview](smilingfriends-close-label.png) and [flight view](flight-smilingfriends-label.png).

The lighting pass darkens the blue and violet cloud field, reduces uniform fill on the dark side of 3D models, narrows the sunlit edge, and adds soft color glows behind the planets. Review the updated [overview](overview-logos.png), [Smiling Friends disk](smilingfriends-close-label.png), [Star Wars](visual-pass-starwars.png), [Minecraft](visual-pass-minecraft.png), [Regular Show](visual-pass-regularshow.png), [South Park](visual-pass-southpark.png), and [Halloween](visual-pass-halloween.png). The glow is drawn behind each model so the surface details remain visible.
