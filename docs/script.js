export class Controller {
  constructor() {
    this.activeStation = null;
    this.activeData = null;
    this.stations = {};
    this.init().then(({stations}) => {
      this.stations = stations;
      this.prep();
    });  
  } 

  async getStation() {
    let currentDistance = 1000000000;
    const latNumber = parseFloat(document.querySelector("#latNumber").value);
    const longNumber = parseFloat(document.querySelector("#longNumber").value);
    for (let noaaId in this.stations) {
      const checkDistance = this.getDistanceFromLatLong(
        this.stations[noaaId].lat, 
        this.stations[noaaId].long,
        latNumber, 
        longNumber, 
      )
      if (checkDistance < currentDistance) {
        this.activeStation = this.stations[noaaId];
        currentDistance = checkDistance;
      }
    }

    let response = await fetch(`/data/predictions/${this.activeStation.noaa_id}.json`);
    if (!response.ok) {
      throw new Error('There was a problem getting the data')
    }
    let data = await response.json();
    console.log(data);

    let stationNameEl = document.querySelector(".stationName");
    stationNameEl.innerHTML = this.activeStation.name;
  }

  getDistanceFromLatLong(lat1,lon1,lat2,lon2) {
    // from: https://stackoverflow.com/a/27943/102401
    var R = 6371; // Radius of the earth in km
    var dLat = this.deg2rad(lat2-lat1);  // deg2rad below
    var dLon = this.deg2rad(lon2-lon1); 
    var a = 
      Math.sin(dLat/2) * Math.sin(dLat/2) +
      Math.cos(this.deg2rad(lat1)) * Math.cos(this.deg2rad(lat2)) * 
      Math.sin(dLon/2) * Math.sin(dLon/2)
      ; 
    var c = 2 * Math.atan2(Math.sqrt(a), Math.sqrt(1-a)); 
    var d = R * c; // Distance in km
    return d;
  }

  deg2rad(deg) {
    return deg * (Math.PI/180)
  }

  prep() {
    const getStationButton = document.querySelector(".getStationButton");
    getStationButton.addEventListener("click", () => { this.getStation.call(this, event) } )
    getStationButton.innerHTML = "Get Station Data"
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

