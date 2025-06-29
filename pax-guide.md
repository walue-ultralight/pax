# Pax Framework Kurulum ve Kullanım Rehberi

Bu rehber, Pax UI framework'ünün kurulumu, derlenmesi ve örnek uygulamaların çalıştırılması için adım adım talimatlar içerir.

## İçindekiler

1. [Gereksinimler](#gereksinimler)
2. [Kurulum](#kurulum)
3. [Örnek Uygulamaları Çalıştırma](#örnek-uygulamaları-çalıştırma)
4. [Yeni Proje Oluşturma](#yeni-proje-oluşturma)
5. [Pax Designer](#pax-designer)
6. [Sorun Giderme](#sorun-giderme)

## Gereksinimler

Pax framework'ünü kullanmak için aşağıdaki yazılımlara ihtiyacınız vardır:

- Rust ve Cargo (en son sürüm)
- wasm32-unknown-unknown hedefi
- wasm-bindgen-cli
- wasm-pack
- pkg-config
- libpango1.0-dev
- libcairo2-dev

## Kurulum

### 1. Rust ve Cargo Kurulumu

Eğer Rust kurulu değilse:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### 2. WebAssembly Hedefini Ekleyin

```bash
rustup target add wasm32-unknown-unknown
```

### 3. wasm-bindgen-cli Kurulumu

```bash
cargo install wasm-bindgen-cli
```

### 4. wasm-pack Kurulumu

```bash
cargo install wasm-pack
```

### 5. Sistem Bağımlılıklarını Yükleyin

Debian/Ubuntu:

```bash
sudo apt-get update
sudo apt-get install pkg-config libpango1.0-dev libcairo2-dev
```

### 6. Pax Repository'sini Klonlayın

```bash
git clone https://github.com/walue-ultralight/pax.git
cd pax
```

## Örnek Uygulamaları Çalıştırma

Pax, çeşitli örnek uygulamalar içerir. Bunları çalıştırmak için:

### Increment Örneği

Bu, tıklandığında sayacı artıran basit bir örnek uygulamadır:

```bash
cd examples/src/increment
./pax run --target=web --no-designer --host 0.0.0.0 --port 8080
```

Uygulama http://localhost:8080 adresinde çalışacaktır.

### Fireworks Örneği

```bash
cd examples/src/fireworks
./pax run --target=web --no-designer --host 0.0.0.0 --port 8080
```

### Calculator Örneği

```bash
cd examples/src/calculator
./pax run --target=web --no-designer --host 0.0.0.0 --port 8080
```

## Yeni Proje Oluşturma

Yeni bir Pax projesi oluşturmak için:

```bash
cd /workspace/pax
cargo run -p pax-cli -- create /path/to/your/new-project
cd /path/to/your/new-project
./pax run --target=web --host 0.0.0.0 --port 8080
```

## Pax Designer

Pax Designer, Pax uygulamalarını görsel olarak tasarlamak için kullanılan bir araçtır. Ancak, derlenmesi ve çalıştırılması uzun sürebilir.

```bash
cd pax-designer
cargo run --features web
```

Not: Pax Designer'ı çalıştırmak için daha güçlü bir bilgisayar gerekebilir, çünkü derleme işlemi oldukça kaynak yoğundur.

## Pax Projesi Yapısı

Tipik bir Pax projesi şu dosyaları içerir:

- `src/lib.rs`: Rust kodu ve mantık
- `src/lib.pax`: Deklaratif UI tanımı
- `Cargo.toml`: Proje bağımlılıkları

### lib.rs Örneği

```rust
#![allow(unused_imports)]

use pax_kit::*;

#[pax]
#[main]
#[file("lib.pax")]
pub struct Example {
    pub ticks: Property<usize>,
    pub num_clicks: Property<usize>,
    pub current_rotation: Property<f64>,
}

impl Example {
    pub fn handle_pre_render(&mut self, _ctx: &NodeContext) {
        let old_ticks = self.ticks.get();
        self.ticks.set((old_ticks + 1) % 255);
    }

    pub fn increment(&mut self, _ctx: &NodeContext, _args: Event<Click>) {
        let old_num_clicks = self.num_clicks.get();
        let new_val = old_num_clicks + 1;
        self.num_clicks.set(new_val);
        self.current_rotation.ease_to(new_val as f64 * 90.0, 120, EasingCurve::OutQuad);
    }
}
```

### lib.pax Örneği

```
<Group x=50% y=50% width=16.83% height=29.55% @click=self.increment rotate={(current_rotation)deg}>
    <Text x=50% y=50% selectable=false text={num_clicks + " clicks"} id=text/>
    <Rectangle fill={rgb(ticks, 75, 255 - ticks)} corner_radii={RectangleCornerRadii::radii(10.00, 10.00, 10.00, 10.00)}/>
</Group>

@settings {
    @pre_render: handle_pre_render
    #text {
        style: {
            font: Font::Web("Roboto", "", FontStyle::Normal, FontWeight::Light)
            font_size: 26px
            fill: WHITE
            align_vertical: TextAlignVertical::Center
            align_horizontal: TextAlignHorizontal::Center
            align_multiline: TextAlignHorizontal::Center
        }
    }
}
```

## Sorun Giderme

### wasm-pack Hatası

Eğer "failed to run wasm-pack" hatası alırsanız:

```bash
cargo install wasm-pack
```

### Derleme Süresi

Pax projelerinin derlenmesi, özellikle ilk kez derlenirken uzun sürebilir. Bu normal bir durumdur, çünkü Rust kodunun WebAssembly'ye dönüştürülmesi gerekir.

### Bağımlılık Hataları

Eğer bağımlılık hataları alırsanız, gerekli sistem paketlerinin yüklü olduğundan emin olun:

```bash
sudo apt-get install pkg-config libpango1.0-dev libcairo2-dev
```

## Kaynaklar

- [Pax GitHub Repository](https://github.com/walue-ultralight/pax)
- [Pax Resmi Websitesi](https://pax.dev/)