# Clyde — Who Will Hit Kenny?

Adaptación del caso concreto **Car Repair / Reparación de coche (6×6)** de Murdoku, Manuel Garand. La geometría, las seis pistas y la solución se transcribieron de las tres capturas proporcionadas. No es un tablero nuevo ni una variante generada.

Las capturas no incluían las reglas generales completas. La ficha oficial del mismo caso confirma una persona por fila/columna y que «junto a» significa una casilla ortogonal dentro de la misma zona: https://murdoku.com/pdf/car-repair-bw.pdf. Ambas condiciones se explican al jugador. Las cruces de la primera captura son descartes, no obstáculos. No quedaron condiciones necesarias sin determinar.

## Transcripción (coordenadas base 1: fila, columna)

El tablero tiene exactamente 36 casillas. Las zonas son irregulares; en particular, el almacén forma una L, no un rectángulo. R=Recepción/Centro, W=Sala de espera/Parada, S=Almacén/Depósito, G=Garaje/Terminal:

```text
R R R W S S
R R R W W S
R R R W W W
R G G G G G
G G G G G G
G G G G G G
```

| Original | Adaptación | Casillas |
|---|---|---|
| Mesa | Edificio | (1,1) |
| Escritorio en L | Edificio en L | (1,3), (2,2), (2,3) |
| Televisor | Edificio de radio | (1,4) |
| Planta | Jardinera | (2,5) |
| Estanterías | Quioscos | (1,6), (3,3), (4,4), (4,5) |
| Sillas | Bancos ocupables | (1,2), (3,4), (3,5) |
| Coches | Autobuses ocupables | (5,2)-(5,3), (6,4)-(6,5) |
| Manchas de aceite | Alcantarillas ocupables | (5,4), (6,2) |

Solo edificios, radio, jardinera y quioscos bloquean casillas: diez en total. Los otros objetos y calles son ocupables por cualquier ficha; las pistas se comprueban únicamente al confirmar.

## Correspondencia lógica

- Vaughn → Kenny: comparte sector con exactamente un chofer, el peligroso. Sigue vivo antes de confirmar.
- Anthony → azul: dentro de un autobús.
- Brock → rojo: sobre una alcantarilla.
- Crystal → verde: sentado en un banco, **en la misma casilla**, no junto a él.
- Diane → negro: es la única ficha de Parada.
- Emilio → blanco: junto a un quiosco, ortogonalmente y en su mismo sector.

Las traducciones EN/ES usan la misma lógica. No se revela una posición correcta ni se da feedback de aciertos parciales. Kenny también empieza en la bandeja sin colocar. Los colores y letras A–E identifican fichas, no su corrección.

## Solución de desarrollo — no mostrada en la interfaz

Kenny (2,1); azul (6,5); rojo (5,4); verde (1,2); negro (3,6); blanco (4,3). El verde es el único chofer en el sector de Kenny. Coincide con la captura resuelta.

`murdoku::solutions()` enumera asignaciones a partir de las reglas, sin consultar la solución guardada. `Case::correct()` evalúa esas restricciones y la identidad elegida, no un contador de posiciones acertadas. `PuzzleRun::confirm_bus()` acepta una sola confirmación completa por intento y entrega el resultado al sistema existente de rescate/muerte, Stan/Kyle y recompensas.

## Interfaz y controles

La pantalla mantiene a Clyde, el marco, las fuentes, el mute y los controles compartidos. El área del puzzle contiene el tablero exacto, seis tarjetas con pistas seleccionables y cinco botones para elegir al chofer peligroso. Clic en tarjeta o ficha colocada selecciona; clic en casilla vacía válida coloca/mueve. «Retirar ficha» o Supr retira la selección. «Vaciar tablero» limpia posiciones y acusación. Confirmar/Enter se habilita únicamente con seis posiciones y un chofer seleccionado.

No hay contador de movimientos, intentos parciales ni límite de tiempo. La respuesta incorrecta usa la derrota existente; reiniciar/reintentar crea una distribución vacía. El antiguo módulo de tráfico se conserva como archivo previo, pero no interviene en el gameplay de Clyde.

## Verificación mínima

```powershell
rustc --edition 2021 -Awarnings tools/southpark/validate_murdoku.rs -o target/validate_murdoku.exe
.\target\validate_murdoku.exe
cargo build --bins
```

El verificador usa las reglas de producción y comprueba geometría, objetos, solución única, equivalencia con las coordenadas de referencia, edición, casillas bloqueadas, confirmación incompleta, seis posiciones más acusación, fallo por distribución incorrecta o acusación incorrecta y bloqueo de confirmaciones posteriores. No ejecuta pruebas visuales ni suites generales. La revisión visual y el audio quedan para prueba manual.
