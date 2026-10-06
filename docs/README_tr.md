# Aeon Engine


[![Lisans: MPL 2.0](https://img.shields.io/badge/License-MPL_2.0-orange.svg)](https://opensource.org/licenses/MPL-2.0)
[![Status](https://img.shields.io/badge/status-in_active_development-green)](https://github.com/aethelisdev/aeon-engine)
[![Rust](https://img.shields.io/badge/rust-v1.99.0-orange?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![YouTube](https://img.shields.io/badge/AeonEngine-FF0000?style=flat&logo=youtube&logoColor=white)](https://youtube.com/@Aeonengine)
[![Instagram](https://img.shields.io/badge/AeonEngine-E4405F?style=flat&logo=instagram&logoColor=white)](https://instagram.com/aeonengine)
[![X / Twitter](https://img.shields.io/badge/AeonEngine-000000?style=flat&logo=x&logoColor=white)](https://x.com/aeonengine)

[English](../README.md) | **Türkçe** | [日本語](README_ja.md) | [简体中文](README_zh.md)

![Aeon Engine Play Mode](../assets/screenshots/aeonengineplaymode.png)

## Aeon Engine Nedir?
Neredeyse tamamen safe Rust ile yazılmış, tam modüler bir oyun motorudur.

## Aeon Engine Neden Oluşturuldu?
Rust'ın güvenliğine ve performansına ilgi duyuyordum ve kendi oyunum için hafif, modüler bir motor istiyordum. Piyasadaki diğer motorlar ihtiyaçlarım için fazla ağırdı, hafif olanlar ise görsel olarak tatmin edici değildi. Bu yüzden hem görsel olarak çok kaliteli hem de hafif bir motor istiyordum. Bu yüzden kendi motorumu geliştirmeye başladım.

## Aeon Engine'in Amacı Nedir?
Geliştirme sürecinde belirlediğim hedefler şunlar:

 - **Modülerlik**: Motorun herhangi bir sistemini tamamen çıkarmak veya değiştirmek istersem çok fazla bağımlılığa maruz kalmadan rahatça değiştirebilme yeteneği 
 
 - **Kullanıcı tercihlerine göre adapte olan bir sistem**: Başlangıçta sistem hiçbir yük olmadan başlar ve render, fizik ve ses gibi modüller kullanıcının ihtiyaçlarına göre etkinleştirilir veya devre dışı bırakılır. Ayrıca düşük donanımlı bilgisayarlar için uzun vadede bu özellikleri RAM'den temizlemelerini sağlayan bir düğme eklemeyi düşünüyorum.
 
 - **Rust kodunu görsel programlamayla birleştirme (RedWrite)**: RedWrite, Rust kodunu görsel programlamayla eş zamanlı olarak birleştirmeyi amaçladığım bir sistemdir. Amacım, iki tarafta yapılan değişikliklerin birbirine anlık ve çift yönlü olarak yansımasıdır.
 
 - **Android cihazlarda editör olarak çalışabilme yeteneği**: Modüler, çalışan bir sistemde iyi sonuçlar ve yüksek performans elde edersem, Aeon motorumu Android cihazlarda doğrudan oyun üretebilen bir editör olarak geliştirerek oyun geliştirmenin Android cihazlarda mümkün olmadığı algısını kırmayı hedefliyorum.

## Mevcut Durum
Aeon Engine şu anda aktif olarak geliştirilmektedir.

Temel sistemlerin çoğu geliştirilmiş durumdadır, ancak API ve özellikler zamanla değişiklik gösterebilir.

## Nasıl Çalıştırılır
Makinenizde Rust kurulu olduğundan emin olun.

```bash
git clone https://github.com/aethelisdev/aeon-engine.git
cd aeon-engine

# Aeon Hub'ı başlat (Proje Yöneticisi & Başlatıcı)
cargo run --release

# veya Editörü doğrudan çalıştır:
cargo run -p ae_engine --release
```

## Tasarım Felsefesi
 
- **Modüler tasarım**
- **Basit editör**
- **Performans**
- **Safe Rust**
- **Gereksiz karmaşıklıktan kaçınmak**
- **Açık Kaynak**

## Lisans

Aeon Engine kaynak kodu **Mozilla Public License 2.0 (MPL 2.0)** altında lisanslanmıştır.

- **Bağımsız geliştiriciler ve oyun yaratıcıları**: Aeon Engine kullanarak ticari, kapalı kaynaklı oyunlar oluşturmakta özgürsünüz. Oyun mantığınızı veya oyun kaynak kodunuzu paylaşmanız gerekmez.
- **Motor modifikasyonları**: Ana motor dosyalarını değiştirirseniz, bu motor değişiklikleri MPL 2.0 lisansı altında paylaşılmalıdır.
- **Ticari ve kurumsal lisanslama**: Özel ticari lisanslama, özel motor çatalları veya kurumsal destek seçenekleri için AethelisDEV ile iletişime geçin.
