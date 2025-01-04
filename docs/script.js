export class Controller {
  constructor() {
    this.activeStationId = "8720291"; // Default to Jacksonville Beach
    this.activeData = null;
    this.stations = {};
    this.init().then(({stations}) => {
      this.stations = stations;
      this.prep();
    });  
  } 

  getState() {
    if (this.stations[this.activeStationId].state != "") {
      return `, ${this.stations[this.activeStationId].state}`
    } else {
      return ""
    }
  }

  async getStation() {
    let response = await fetch(`/data/predictions/${this.activeStationId}.json`);
    if (!response.ok) {
      throw new Error('There was a problem getting the data')
    }
    let data = await response.json();
    const maxMinutesBefore = parseInt(this.maxMinutesBeforeEl.value, 10);
    const maxMinutesAfter = parseInt(this.maxMinutesAfterEl.value, 10);
    const year = parseInt(this.yearEl.value, 10);
    let items = [];
    data.low_tides.forEach((item) => {
      if (item.sunrise[0] == year) {
        if (
          item.sunrise[7] >= (maxMinutesBefore * -1) 
            &&
          item.sunrise[7] <= maxMinutesAfter) {
          items.push(item)
        }
      }
    })
    this.outputItems(items)
  }

  outputItems(items) {
    let outputStuff = [];
    outputStuff.push(`<h2>${this.yearEl.value} - ${this.stations[this.activeStationId].name}${this.getState()}</h2>`)
    outputStuff.push(`<table class="data-table"><thead>`)
    outputStuff.push(`<tr class="tideLine"><th class="left">Date</th><th>Sunrise</th><th>Low Tide</th><th>Diff</th></tr>`);
    outputStuff.push(`</thead><tbody>`)
    items.forEach((item) => {
      outputStuff.push(`<tr class="tideLine">`)
//      outputStuff.push(`<td>${item.tide_local_year}-${this.pad2(item.tide_local_month)}-${this.pad2(item.tide_local_day)} (${item.tide_local_day_string})</td>`)
      outputStuff.push(`<td>${item.tide[5]}</td>`)
      outputStuff.push(`<td class="center">${item.sunrise[3]}:${this.pad2(item.sunrise[4])}</td>`)
      outputStuff.push(`<td class="center">${item.tide[3]}:${this.pad2(item.tide[4])}</td>`)
      outputStuff.push(`<td class="center">${Math.abs(item.sunrise[5])}:${this.pad2(item.sunrise[6])}`)
      if (item.sunrise[7] >= 0 ) {
        outputStuff.push(` [+]`)
      } else {
        outputStuff.push(` [-]`)
      }
      outputStuff.push(`</td>`)
      outputStuff.push(`</tr>`)
    });
    const dataEl = document.querySelector(".stationData");
    dataEl.innerHTML = outputStuff.join("");
  }

  pad2(num) {
    return num.toString().padStart(2, '0')
  }

  getDistanceFromLatLong(lat1, lon1, lat2, lon2) {
    // from: https://stackoverflow.com/a/27943/102401
    var R = 6371; // Radius of the earth in km
    var dLat = this.deg2rad(lat2-lat1); 
    var dLon = this.deg2rad(lon2-lon1); 
    var a = 
      Math.sin(dLat/2) * Math.sin(dLat/2) +
      Math.cos(this.deg2rad(lat1)) * Math.cos(this.deg2rad(lat2)) * 
      Math.sin(dLon/2) * Math.sin(dLon/2); 
    var c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a)); 
    var d = R * c;
    return d;
  }

  deg2rad(deg) {
    return deg * (Math.PI/180)
  }

  prep() {
    this.maxMinutesBeforeEl = document.querySelector("#maxMinutesBefore");
    this.maxMinutesBeforeEl.addEventListener("change", (event) => this.getStation.call(this, event))
    this.maxMinutesAfterEl = document.querySelector("#maxMinutesAfter");
    this.maxMinutesAfterEl.addEventListener("change", (event) => this.getStation.call(this, event))
    this.yearEl = document.querySelector("#year");
    this.yearEl.addEventListener("change", (event) => this.getStation.call(this, event))
    var map = L.map('map').setView([38, -100], 3);
    L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
        maxZoom: 19,
        attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
    }).addTo(map);
    for (let station in this.stations) {
      const marker = L.marker(
        [this.stations[station].lat, this.stations[station].long
      ]).addTo(map);
      marker.noaa_id = this.stations[station].noaa_id;
      marker.addEventListener("click", (event) => this.switchStation.call(this, event))
    }
    this.getStation()
  }

  switchStation(event) {
    const el = event.target;
    this.activeStationId = el.noaa_id;
    this.getStation();
  }

  async init() {
    let response = await fetch('/data/stations.json');
    if (!response.ok) {
      throw new Error('There was a problem getting the data')
    }
    let data = await response.json();
    let stations = {};
    data.stations.forEach((station) => {
      stations[station.noaa_id] = {
        "noaa_id": station.noaa_id,
        "name": station.name,
        "lat": station.lat,
        "long": station.long,
        "state": station.state,
      }
    })
    return { stations }
  }

}

