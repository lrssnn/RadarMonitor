window.Vue = Vue;

let app = new Vue({
    el: '#app',
    data: {
        msg: 'Working',
        slow: false,
        med: true,
        fast: false,
        bg: ["res/IDR042.background.png","res/IDR043.background.png", "res/IDR044.background.png"],
        lc: ["res/IDR042.locations.png","res/IDR043.locations.png", "res/IDR044.locations.png"],
        img: "img/IDR044/0.png",
        index_s: 0,
        index_m: 0,
        index_f: 0,
        zoom: 2,
        images: [[""],
          ["img/IDR042/0.png", "img/IDR042/1.png", "img/IDR042/2.png",
            "img/IDR042/3.png", "img/IDR042/4.png", "img/IDR042/5.png",
            "img/IDR042/6.png", "img/IDR042/7.png", "img/IDR042/8.png",
            "img/IDR042/9.png", ],
            ["img/IDR043/0.png", "img/IDR043/1.png", "img/IDR043/2.png",
                "img/IDR043/3.png", "img/IDR043/4.png", "img/IDR043/5.png",
                "img/IDR043/6.png", "img/IDR043/7.png", "img/IDR043/8.png",
                "img/IDR043/9.png", ],
            ["img/IDR044/0.png", "img/IDR044/1.png", "img/IDR044/2.png",
                "img/IDR044/3.png", "img/IDR044/4.png", "img/IDR044/5.png",
                "img/IDR044/6.png", "img/IDR044/7.png", "img/IDR044/8.png",
                "img/IDR044/9.png", ]],

    },
    computed: {
        index: function() {
            if (this.slow) {
                return this.index_s;
            } else if (this.med) {
                return this.index_m;
            } else {
                return this.index_f;
            }
        }
    },
    methods: {
        doSomething(){
            console.log("Good job");
        },
        set_slow() {
            // Synchronise index to prevent jumps
            if (this.med) {
                this.index_s = this.index_m;
            } else if (this.fast) {
                this.index_s = this.index_f;
            }

            this.slow = true;
            this.med = false;
            this.fast = false;
        },
        set_med() {
            if (this.slow) {
                this.index_m = this.index_s;
            } else if (this.fast) {
                this.index_m = this.index_f;
            }
            this.slow = false;
            this.med = true;
            this.fast = false;
        },
        set_fast() {
            if (this.med) {
                this.index_f = this.index_m;
            } else if (this.slow) {
                this.index_f = this.index_s;
            }
            this.slow = false;
            this.med = false;
            this.fast = true;
        },
        set_zoom(zoom) {
            this.zoom = zoom;
        },
        get_listing() {
          console.log("Get Listing:");
          var request = new XMLHttpRequest();
          var v = this;
          request.onreadystatechange = function() {
            if (request.readyState == 4 && request.status == 200){
                console.log(JSON.parse(request.responseText))

                v.images = JSON.parse(request.responseText);
                let refresh = new Date(v.images[0][0] * 1000)
                console.log("Refresh Time: ", refresh);
                // Set the listing to refresh at the specified time
                let delay = (v.images[0][0] * 1000) + 5000 - Date.now();
                console.log("Refeshing in: ", delay, "ms");
                window.setTimeout(v.get_listing, delay);
              }
          }
          request.open("GET", "http://localhost:8000/listing", true);
          request.send(null);
        }
    },
    created: function() {
        window.setInterval(() => {
            this.index_s = (this.index_s + 1) % 30;
        }, 500);

        window.setInterval(() => {
            this.index_m = (this.index_m + 1) % 30;
        }, 200);

        window.setInterval(() => {
            this.index_f = (this.index_f + 1) % 30;
        }, 80);

        this.get_listing();
    }
});
