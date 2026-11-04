// SIPIL: berkas ini menggantikan sw.js lama di /sipilcad/sw.js.
//
// Dulu /sipilcad/ adalah aplikasi web itu sendiri, dengan service worker ber-scope /sipilcad/ yang menyimpan
// cache aplikasi (wasm ~55 MiB) di peramban pengguna. Sekarang /sipilcad/ adalah halaman landing dan aplikasi
// ada di /sipilcad/web/ (service worker-nya sendiri, scope /sipilcad/web/). Peramban memeriksa pembaruan
// service worker di URL yang SAMA saat pengguna lama datang, lalu memasang berkas ini. Tugasnya satu kali:
// hapus cache aplikasi lama, lalu mencabut dirinya. Tidak ada handler fetch, jadi permintaan tidak dicegat.
//
// Berkas ini aman dihapus setelah beberapa bulan, ketika pengguna lama sudah sempat berkunjung.
self.addEventListener("install", () => self.skipWaiting());

self.addEventListener("activate", (event) => {
  event.waitUntil(
    (async () => {
      const names = await caches.keys();
      await Promise.all(names.filter((name) => name.startsWith("ocs-app-")).map((name) => caches.delete(name)));
      await self.registration.unregister();
    })(),
  );
});
