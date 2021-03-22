package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"

	"github.com/gorilla/websocket"
	"github.com/nats-io/nats.go"
)

var upgrader = websocket.Upgrader{
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
} // use default options

var connections = make([]*websocket.Conn, 0)

func passthrough(w http.ResponseWriter, r *http.Request) {
	c, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Print("upgrade:", err)
		return
	}
	connections = append(connections, c)
	//for {
	//	mt, message, err := c.ReadMessage()
	//	if err != nil {
	//		log.Println("read:", err)
	//		break
	//	}
	//	log.Printf("recv: %s", message)
	//	err = c.WriteMessage(mt, message)
	//	if err != nil {
	//		log.Println("write:", err)
	//		break
	//	}
	//}
}

func main() {
	nc, err := nats.Connect("nats://167.99.232.215:4222")
	if err != nil {
		log.Fatal(err)
	}
	defer nc.Close()

	// Simple Async Subscriber
	nc.Subscribe(">", func(m *nats.Msg) {
		fmt.Printf("Received a message: %s %s\n", m.Subject,	 string(m.Data))
		var data map[string]interface{}
		if err := json.Unmarshal(m.Data, &data); err != nil {
			log.Println("Error unmarshaling data", err)
			return
		}

		distance, ok := data["distance"].(float64)
		if ok != true {
			return
		}
		doorState := "closed"
		if distance > 80 {
			doorState = "opened"
		}

		data["doorState"] = doorState

		rawPayload, err := json.Marshal(data)
		if err != nil {
			log.Println("Error marshaling data", err)
			return
		}

		for index, conn := range connections {
			fmt.Println("Sending a message")

			if err := conn.WriteMessage(websocket.TextMessage, rawPayload); err != nil {
				log.Println("write:", err)
				conn.Close()
				connections = append(connections[:index], connections[index+1:]...)
			}
		}
	})

	http.HandleFunc("/", passthrough)
	log.Fatal(http.ListenAndServe("localhost:8080", nil))
}
