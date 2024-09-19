## Models

An example of `ExecutionContext` JSON for convenience:

```json
        {
                    "variables": {
                        "map": {
                            "user": {
                                "type": "map",
                                "value": {
                                    "should_display": {
                                        "type": "bool",
                                        "value": true
                                    },
                                    "some_value": {
                                        "type": "uint",
                                        "value": 7
                                    }
                                }
                            }
                        }
                    },
                    "computed" : {
                      "daysSinceEvent": [{
                                        "type": "string",
                                        "value": "event_name"
                                    }]
                    },
                    "expression": "platform.daysSinceEvent(\"test\") == user.some_value"
        }
```