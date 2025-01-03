export class Controller {
  constructor() {
    this.init();  
  } 

  init() {
    console.log("init controller")
    this.getStationButton = document.createElement("button")
    this.getStationButton.addEventListener("click", this.getStation)
    this.getStations().await
  }

  async getStations() {
    console.log("Getting stations")
//    const 
    fetch('/data/stations.json')
      .then((response) => {
        if (!response.ok) {
          throw new Error('There was a problem getting the data')
        }
        return response.text()
      })
      .then((data) => console.log(data))
      .catch((error) => {
        console.error('Fetch Error: ', error)
      })
  }

  getStation() {
    console.log("Caught Get station")
  }
}

