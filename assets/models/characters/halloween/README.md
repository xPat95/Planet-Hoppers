# Personajes del planeta Halloween

Diseño: [guía de Cami](../../../../docs/cami-guidelines.md), sección 2.

## Brujas

Archivo: **[witch.blend](witch.blend)** (Blender 5.2). Brujas de ilustración de moda: cabeza chica, piernas muy largas y colores planos sin brillo. Piel verde, sombrero negro de ala ancha con la punta curvada y cinta morada, mini vestido negro con el borde en picos, botas negras de tacón hasta media pierna, medias a rayas blancas y negras hasta arriba de la rodilla y el pelo como un solo panel ondulado. La cara solo tiene dos ojitos verdes que brillan.

| Modelo | Pose | Origen | Triángulos |
| --- | --- | --- | ---: |
| `HW_Witch_Solo_A` | Bailando sola: cadera de lado, rodilla cruzada en punta de pie, brazos doblados junto a la cabeza | Bajo los tacones | ~1 460 |
| `HW_Witch_Solo_B` | Bailando sola: brazos de cactus y una rodilla levantada | Bajo los tacones | ~1 460 |
| `HW_Witch_Solo_C` | Bailando sola: estocada de disco, un brazo señalando arriba y el otro cruzado | Bajo los tacones | ~1 460 |
| `HW_Witch_Pair` | Dos brujas frente a frente, de la mano | Bajo los tacones | ~3 180 |
| `HW_Witch_Crawl` | Gateando, cabeza levantada | Bajo las manos y rodillas | ~1 460 |
| `HW_Witch_Climb` | Colgada de un borde, brazos arriba | En las manos | ~1 460 |
| `HW_Witch_Peek` | Asomándose boca abajo, como por una ventana | Bajo el pecho | ~1 460 |
| `HW_Witch_Sit` | Sentada en un borde, columpiando las piernas | Bajo la cola | ~2 190 |

Miden ~1.25 unidades; el planeta las usa como copias a escala 1.9. **No editar el `.blend` a mano:** se regenera con [build_witch.py](../../../../tools/blender/halloween/build_witch.py).

## Diablitos

Archivo: **[devil.blend](devil.blend)** (Blender 5.2). Mismo estilo suave y simple que las brujas (cuerpo y brazos con muchos anillos que se afinan de a poco, codos y hombros redondeados, pulgar): rojo carmesí vivo (distinto del rojo ladrillo del cráneo, con un poco de brillo propio), ojos cerrados felices, dos cuernitos casi rectos del mismo rojo y hombros redondeados que unen los brazos al cuerpo. Solo la mitad de arriba, porque bailan saliendo de las grietas: el origen va en la cintura, que queda hundida en la grieta. Pocas caras, porque van muchos.

| Modelo | Pose de baile | Triángulos |
| --- | --- | ---: |
| `HW_Devil_ArmsUp` | Los dos brazos arriba, torso ladeado | ~1 090 |
| `HW_Devil_Cactus` | Brazos de cactus | ~1 090 |
| `HW_Devil_Point` | Disco: un brazo señalando arriba, el otro cruzado | ~1 090 |
| `HW_Devil_Sway` | Meneándose, un brazo arriba y la mano en la cintura | ~1 090 |

El planeta los usa como copias a escala ~4.2. **No editar el `.blend` a mano:** se regenera con [build_devil.py](../../../../tools/blender/halloween/build_devil.py).
