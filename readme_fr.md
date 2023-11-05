# DHT PIO Rust Library

[![crates.io](https://img.shields.io/crates/v/dht-pio)](https://crates.io/crates/dht-pio) [![MIT](https://img.shields.io/github/license/jnthbdn/rs-dht-pio)](https://opensource.org/licenses/MIT) [![GitHub](https://img.shields.io/badge/github-%23121011.svg?style=for-the-badge&logo=github&logoColor=white)](https://github.com/jnthbdn/rs-dht-pio)


_[english version](readme.md)_


## Pourquoi ?
Les DHT (22 ou 11) utilisent un protocole 1-Wire, qui n'est pas compatible avec le protocole du même nom de [Dallas Semicondutors](https://en.wikipedia.org/wiki/1-Wire). Le [Raspberry Pico](https://www.raspberrypi.com/products/raspberry-pi-pico/) (comme d'autres microcontrôleurs) ne possède pas de périphérique dédié à ce protocole. 

De nombreuses crates existes pour utiliser le DHT via un pin digital, mais après en avoir testée plusieurs, leur fonctionnement n'est pas fiable. Le problème vient de l'implémentation de la [embedded_hal](https://crates.io/crates/embedded-hal) par [rp2040_hal](https://crates.io/crates/rp2040-hal). La manipulation de l'état et de la direction d'une pin, prend trop de temps (j'ai pu mesure entre 2µs et 6µs suivant l'action demandée). Venant, entre autre, de l'impossibilité de mettre une pin en drain ouvert ("open drain"), ce qui nécessite de "simuler" ce comportant.

## Le PIO ❤️
Le chip RP2040 (utilisé pour le Pico), possède un périphérique un peu atypique nommée PIO (Programmable Input/Output), [Chapitre 3 de la DataSheet](https://datasheets.raspberrypi.com/rp2040/rp2040-datasheet.pdf). En simplifiant, le principe est de pouvoir faire tourner un petit programme (32 instruction max), qui s'éxécutera de manière indépendante. Il peut manipuler les GPIO et partager des informations avec le programme principal.

Le PIO se programme à l'aide d'un assembleur nommé `pioasm`, il ne comporte que quelques instructions très basiques. Le plus intéressant est que chaque instruction prends (en général), 1 cycle pour s'exécute. De plus il est possible de diviser la clock à laquelle le programme s'éxécute. Dans le notre cas, on disive la clock principale, de 125 MHz, par 125, ce qui nous donne une instruction par microsecondes.

## Usage
Dans un premier temps créer récupérer les objets PIO
```rust
let (dht_pio, dht_sm, _, _, _) = pac.PIO0.split(&mut pac.RESETS);
```
Pour créer un nouvelle objet:
- DHT22  
  ```rust
  let mut dht = Dht22::new(dht_pio, dht_sm, pins.gpio0.into_function());
  ```
- DHT11
  ```rust
  let mut dht = Dht11::new(dht_pio, dht_sm, pins.gpio0.into_function());
  ```

Lire les données:
```rust
let dht_data = dht.read(&mut delay);
```

NB: `read` renvoi un `Result<DhtResult, DhtError>`.

### DHT22 Type 2 🧐
Il semble qu'il existe deux versions de DHT22. Je n'ai rien trouvé de vraiment probant, mais ce qui est certain c'est que tous les DHT22 n'ont pas le même format de donnée... Dans un cas le format est le même que présenté dans (quasi) toutes les datasheet, à savoir le bit de poids fort est à l'état `1` si le nombre est négatif, **mais** la représentation binaire de la valeur absolue de la température n'en est pas changée. Par exemple:
  - `0000 0000 0110 1001` = 105 soit 10.5°C
  - `1000 0000 0110 1001` = 32873 soit -10.5°C

C'est comme cela que la struct `Dht22` va "décoder" les données en provenance du capteur.
Or je suis tombé sur des capteurs qui ne fonctionnaient pas du tout comme cela. Mais de manière (au final) plus logique. Puisque les données sont représentées en [**complément à deux**](https://fr.wikipedia.org/wiki/Compl%C3%A9ment_%C3%A0_deux). Dans ce cas il faut utiliser `Dht22Type2`. Par exemple:
  - `0000 0000 0110 1001` = 105 soit 10.5°C
  - `1111 1111 1001 0111` = 65431 soit -10.5°C

Pour simplifier si votre capteur est un DHT22 mais que les valeurs ne semblent pas cohérentes (valeurs négatives) alors essayez le "Type 2" (et si vraiment rien ne marche, ouvrez une issue 😉).

## Support
### Board
Pour le moment le crates n'a été testé que sur un Raspberry Pico.

### DHT
✅ DHT22  
❔ DHT11

## TODO
- [ ] Finir le Readme
- [x] Ajouter la lecture du CRC
- [x] Vérifier le CRC
- [x] Support du DHT11
- [ ] Tester DHT11
- [ ] Documenter le code

## Remerciement
 <img src="https://avatars.githubusercontent.com/u/10778792?v=4" style="width: 40px; border-radius: 50%; vertical-align: middle;" /> [Geir Ertzaas (grukx)](https://github.com/grukx), pour avoir découvert (trop?) de bugs.