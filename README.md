# ecowitt2db

Emulates an ecowitt server endpoint. Converts the values to metric and stores the weather data into InfluxDB.

## Dealing with InfluxDB historical data

**TODO**

To downsample historical data in InfluxDB v2, you should use InfluxDB tasks and Flux queries to aggregate and store down-sampled data in separate buckets with longer retention policies. This approach allows you to keep high-resolution raw data for a shorter period and store summarized, lower-resolution data for longer, saving storage space and improving query performance for historical analysis.

## POST Data format

Example HTTP Form data posted by Sainlogic WS3500:

```json
{
  "freq": "868M",
  "tempf": "86.7",
  "maxdailygust": "4.47",
  "windspeedmph": "0.67",
  "baromabsin": "29.241",
  "winddir": "295",
  "rainratein": "0.000",
  "solarradiation": "446.50",
  "weeklyrainin": "0.110",
  "eventrainin": "0.000",
  "dailyrainin": "0.000",
  "vpd": "0.795",
  "stationtype": "EasyWeatherPro_V5.2.2",
  "runtime": "2389",
  "monthlyrainin": "0.201",
  "tempinf": "73.6",
  "wh65batt": "0",
  "heap": "22632",
  "hourlyrainin": "0.000",
  "windgustmph": "1.12",
  "humidity": "38",
  "dateutc": "2025-07-19 15:25:07",
  "humidityin": "59",
  "uv": "4",
  "yearlyrainin": "0.201",
  "interval": "60",
  "PASSKEY": "12345678901234567890123456789012",
  "model": "WS2900_V2.02.04",
  "totalrainin": "0.201",
  "baromrelin": "29.784"
}
```

For convenience an example curl request:

```bash
curl -X POST \
-d "freq=868M" \
-d "tempf=86.7" \
-d "maxdailygust=4.47" \
-d "windspeedmph=0.67" \
-d "baromabsin=29.241" \
-d "winddir=295" \
-d "rainratein=0.000" \
-d "solarradiation=446.50" \
-d "weeklyrainin=0.110" \
-d "eventrainin=0.000" \
-d "dailyrainin=0.000" \
-d "vpd=0.795" \
-d "stationtype=EasyWeatherPro_V5.2.2" \
-d "runtime=2389" \
-d "monthlyrainin=0.201" \
-d "tempinf=73.6" \
-d "wh65batt=0" \
-d "heap=22632" \
-d "hourlyrainin=0.000" \
-d "windgustmph=1.12" \
-d "humidity=38" \
-d "dateutc=2025-07-19 15:25:07" \
-d "humidityin=59" \
-d "uv=4" \
-d "yearlyrainin=0.201" \
-d "interval=60" \
-d "PASSKEY=12345678901234567890123456789012" \
-d "model=WS2900_V2.02.04" \
-d "totalrainin=0.201" \
-d "baromrelin=29.784" \
  http://localhost:3000/data/report/
```
