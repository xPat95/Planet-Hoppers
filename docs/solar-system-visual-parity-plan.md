# Solar system visual parity: Blender references and game renderer

Status: first lighting milestone implemented. The camera/material audit, planet-specific fill, and targeted Regular Show and Halloween lights are complete. Shadow maps, emissive bloom, tone mapping, and quality presets remain proposed. See [updated captures](references/solar-camera/README.md).

## What the references actually show

| Example | Blender reference | Current game capture | Main visible gap |
| --- | --- | --- | --- |
| Halloween | [Three-quarter render](references/halloween/halloween_v01_three_quarter.png) | [Close view](references/solar-camera/visual-pass-halloween.png) | The red surface is flatter in game; the eye sockets, lava and black drips have less depth and separation. |
| Smiling Friends | [Three-quarter render](references/smilingfriends/smilingfriends_v01_three_quarter.png) | [Close view](references/solar-camera/smilingfriends-close-label.png) | Buildings and trees merge into the dark side; the disk is seen from a different angle and at a smaller screen size. |
| Regular Show | [Approved dome render](references/regularshow/Domo_Espacial_Regular_Show.png) | [Latest close view](references/solar-camera/visual-pass-regularshow.png) | The park under the dome is much darker and the glass/grid is hard to read. An [older runtime-model render](references/regularshow/regularshow_runtime_near.png) already looks much closer to the approved image. |
| Minecraft | [Textured reference](references/minecraft/minecraft_planet_textured_v02.png) | [Latest close view](references/solar-camera/visual-pass-minecraft.png) | The top blocks and trees lose color and definition. |
| South Park | [Three-quarter render](references/southpark/southpark_v03_three_quarter.png) | [Latest close view](references/solar-camera/visual-pass-southpark.png) | The bright snow, houses and roads become gray and subdued. |
| Star Wars | [Approved render](references/starwars/starwars_approved.png) | [Latest close view](references/solar-camera/visual-pass-starwars.png) | The sphere's panel lines survive, but the overall brightness and tonal gradient are much lower. |

These are not matched camera comparisons. The Smiling Friends Blender image uses an orthographic presentation camera, while solar-system gameplay uses a moving perspective camera. A camera-matched comparison must come before judging shader changes.

## Why they differ

1. **Lighting rigs:** the Halloween Blender script creates three colored SUN lights plus a strong orange point light inside the skull (`tools/blender/halloween/build_halloween_planet.py`). The Smiling Friends script creates four presentation lights (`tools/blender/smilingfriends/build_smilingfriends_planet.py`). The solar system calls `SpaceShader::begin_frame` with one Sun and ambient 0.16. One central light cannot reproduce highlights from several directions or the skull's internal glow.
2. **Surface response:** `src/graphics/shaders/space.fs` uses a hand-built diffuse term, one specular highlight based on roughness, and a simple rim term. Blender's material nodes and Eevee lighting yield different glossy highlights, especially on Halloween's wet red surface. The runtime reads glTF emissive and roughness, but it does not reproduce the entire Blender material graph.
3. **No surface shadows from the Sun:** the runtime shader has no shadow map or ambient occlusion pass. Buildings, trees and facial structures therefore do not gain the contact and cast shadows that give the Blender images depth.
4. **Different glow:** the current game draws additive circular gradients behind the Sun and planets. These soften a silhouette but cannot bloom only the actual bright lava, windows, stars or glossy spots. The Sun and sky also bypass a shared HDR/tone-mapping pipeline.
5. **Presentation:** Blender renders are tightly framed with controlled backgrounds; the game must look good from any angle, at multiple distances, with a moving star field and HUD. The current solar lighting is deliberately distance-limited, so outer planets receive less light than a studio-lit Blender render.
6. **Regular Show has a transparency problem to investigate:** its Blender reference was rendered in Cycles with 64 samples, denoising, four broad area lights and up to 20 transparent bounces (`tools/blender/regularshow/crear_domo.py`). The game sorts transparent *parts* within a model and disables depth writes for them, but it does not trace light through the dome or reproduce those bounces. Verify whether the dome's material/alpha and draw order additionally darken the park. This is a hypothesis to test, not yet an established shader bug.

The older Regular Show runtime-model image is useful evidence: a version of the exported asset can look close to the approved composition without new geometry. The latest darker solar-system pass is therefore a likely contributor to the loss of readability, though the two captures also use different renderers and cameras. It should be measured with matched captures before reverting any value.

## Options

| Option | Work | Expected result | Cost / limitation |
| --- | --- | --- | --- |
| A. Art-direction and material pass **(first choice)** | Camera-matched captures, per-planet fill/exposure, a dome transparency audit, material checks, and selective recovery of color; keep the current renderer. | Highest immediate benefit for Regular Show, Minecraft and South Park, with little GPU cost. Can restore the park's readability before adding effects. | Still lacks Blender-style cast shadows, transparent light bounces and material-aware bloom. |
| B. Hybrid real-time lighting **(second stage, if needed)** | A plus a small number of local lights for special features, selective shadowing, material-aware highlights, and bloom from bright pixels at reduced resolution. | Closer depth and glow while preserving free flight and dynamic camera; particularly useful for Halloween. | More shader/render-target work; requires GPU profiling and quality levels for teammates. Bloom does not fix dark interiors. |
| C. Full PBR-style pipeline | Rework material import, linear/HDR rendering, physically based lights, shadow maps, image-based fill, tone mapping and postprocessing. | Highest potential fidelity and consistent materials. | Largest scope and risk: model/material conversion, transparent objects, performance, and visual retuning across every planet and ship. Blender parity is still not automatic. |
| D. Pre-rendered views/impostors | Render specific planet angles in Blender and blend between them at distance, switching to live geometry near approach. | Excellent distant showcase images with low shading cost. | Free rotation reveals view swaps; alignment and close-up handoff are difficult. Better as a distant LOD technique than the main solution. |

## Recommended sequence: option A, then targeted parts of B

1. **Capture a fair baseline.** Prioritize Regular Show, then Halloween, Minecraft, South Park, Smiling Friends and Star Wars. Add debug comparison cameras that match each Blender reference's direction, framing and model rotation; use the same output resolution. Include both the previous and current game lighting for Regular Show. Measure frame time on the project's Intel graphics and at least one teammate's GPU; do not use the one-frame FPS number in a smoke screenshot as a benchmark.
2. **Audit Regular Show's dome and material transfer.** Inspect the exported GLBs for base color, roughness, metallic/emissive values, alpha mode and missing textures. Render Regular Show with the dome temporarily hidden, then with its material forced to a known neutral transparent state. If the park remains dark without the dome, lighting is the primary cause; if it brightens, correct transparency, sorting or depth behavior. Do the same kind of focused material test for Halloween's red gloss and lava. Keep the hangar's studio mode separate.
3. **Restore readability with planet-specific lighting profiles.** Keep the central Sun as the main direction but separate its contribution from a modest, camera-independent fill. Regular Show's interior needs soft light that passes the dome visually; Minecraft's top and South Park's snow need their intended color range. Star Wars should retain a strong light-to-dark gradient, and Halloween should stay frightening and red. Avoid a global brightness increase that washes out every planet. Compare the last solar-system dimming pass with each approved reference before settling these values.
4. **Add local light only where it has a clear purpose.** Halloween gets localized orange light from openings and a cooler edge; Regular Show may need a bounded interior fill that follows the planet. These lights should be attached to the planet, not the camera, and should fade by distance. Set an upper limit on active local lights.
5. **Add depth selectively.** Start with one shadowed directional light or a cheaper contact-shadow/AO treatment for close planet views. Compare whether major shapes gain enough depth to justify the cost. Reduce or disable it at overview distances. Do not expect screen-space AO alone to reveal hidden geometry.
6. **Replace generic glow where it matters.** Render bright emissive areas to a lower-resolution texture, blur that texture and composite it over the scene. Limit bloom so lava, magical light and the Sun glow without washing out labels, stars or the entire red skull. Preserve a small silhouette halo if it helps navigation. Regular Show should be judged on interior legibility, not bloom strength.
7. **Unify brightness control.** Render lighting in linear space, then apply one exposure/tone-mapping step and convert to display color. Tune Regular Show and Halloween first, then inspect every other planet and the selected ship; materials should not jump in brightness on transition to the hangar.
8. **Provide quality settings.** At minimum: low (option A, no shadow/bloom), medium (local light and reduced-resolution bloom), high (shadows and fuller bloom). Keep one visual target per setting and measure average and worst-frame time during overview, close approach and rotation.

## Acceptance checks

- Halloween: skull silhouette, eye and mouth voids, black drips and lava remain distinct from front, three-quarter and back views. Gloss reads as wet red without turning into a flat red disk.
- Smiling Friends: central building, roads, trees and cloud ring remain distinct across a full rotation; color is recognizable without becoming uniformly lit.
- Regular Show: the park house, trees, paths and lawn remain visible through the dome from front and three-quarter views; dome grid and transparency remain legible without turning the whole planet into a bright lamp.
- Minecraft and South Park: top terrain and snow/houses recover their approved colors without flattening the shaded sides.
- Other four planets and all ships remain legible; no large brightness jumps during approach, orbit or departure.
- At the team's agreed quality setting, frame times remain stable on the weakest target hardware. Effects can be disabled individually for diagnosis.
- Reference captures are refreshed in `docs/references/solar-camera/` after each visual milestone.

## Decision

Recommended: start with the camera/material audit and **option A**. Regular Show is the first acceptance target because it is Cami's favorite and the previous runtime-model image proves much of its look is achievable without new geometry. Only then add the useful parts of **option B** in this order: local lighting, selective depth, emissive bloom, unified tone mapping. Stop after each step to compare captures and frame time. Option C should only be considered if these comparisons still show a large gap and the team has time for a renderer rewrite.
