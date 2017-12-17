window.Vue = Vue;

let app = new Vue({
    el: '#app',
    data: {
        slow: false,
        med: true,
        fast: false,
        img: "img/IDR044/0.png",
        index_s: 0,
        index_m: 0,
        index_f: 0,
	length: 0,
        zoom: 2,
        bg: ["", 
            "img/IDR042.background.png",
            "img/IDR043.background.png", 
            "img/IDR044.background.png"],
        lc: ["", 
            "img/IDR042.locations.png",
            "img/IDR043.locations.png", 
            "img/IDR044.locations.png"],
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
        // Return whichever index is active
        index: function() {
	    let index;
            if (this.slow) {
                index = this.index_s;
            } else if (this.med) {
                index = this.index_m;
            } else {
                index = this.index_f;
            }
	    return index;
        },
    },
    methods: {
	image_x: function() {
            // This represents the width property on the images
	    // in order to centre the image.
	    return ('left:' + (window.innerWidth/2 - 256) + 'px');
	},
        // Each 'set' method ensures that only the desired speed is active
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
        // Contact the server for a fresh image listing. Automatically reschedules itself
        // to call again based on the time supplied in the listing.
        get_listing() {
          // Create the request object
          var request = new XMLHttpRequest();
          // Required as 'this' is aliased in the callback
          var v = this;
          // This callback is called after the response is received
          request.onreadystatechange = function() {
            // Only execute once the request has successfully returned
            if (request.readyState == 4 && request.status == 200){
                // Set the Vue property to the new array
                v.images = JSON.parse(request.responseText);
                // Set the listing to refresh at the specified time
                // (v.images[0][0] is the number of seconds since the Unix epoch at the
                //   time that the next refresh will happen. Convert to millis, add 5
                //   seconds and subtract Date.now() to get the number of millis until
                //   that moment plus 5 secs to ensure server refresh is complete)
                let delay = (v.images[0][0] * 1000) + 5000 - Date.now();
                window.setTimeout(v.get_listing, delay);

		// Pull out the length of the array
	        v.length = v.images[0][1];
              }
          }
          // Send the request
          request.open("GET", "http://localhost:8000/listing", true);
          request.send(null);
        }
    },
    // When the vue instance is created, set the interval functions which drive the
    // indices, and get initialise the images from the server.
    created: function() {
        window.setInterval(() => {
            this.index_s = (this.index_s + 1) % this.length;
        }, 500);

        window.setInterval(() => {
            this.index_m = (this.index_m + 1) % this.length;
        }, 200);

        window.setInterval(() => {
            this.index_f = (this.index_f + 1) % this.length;
        }, 80);

        this.get_listing();
    }
});
