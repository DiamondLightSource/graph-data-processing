{{/*
Expand the name of the chart.
*/}}
{{- define "datasets.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this (by the DNS naming spec).
If release name contains chart name it will be used as a full name.
*/}}
{{- define "datasets.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart name and version as used by the chart label.
*/}}
{{- define "datasets.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels
*/}}
{{- define "datasets.labels" -}}
helm.sh/chart: {{ include "datasets.chart" . }}
{{ include "datasets.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels
*/}}
{{- define "datasets.selectorLabels" -}}
app.kubernetes.io/name: {{ include "datasets.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
Create the name of the service account to use
*/}}
{{- define "datasets.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "datasets.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Create the database URL string
*/}}
{{- define "datasets.databaseURL" -}}
{{- $host_parts := urlParse .Values.database.host }}
{{- $raw_user_info := printf "%s:$DATABASE_PASSWORD" .Values.database.user }}
{{- $url_parts := set $host_parts "userinfo" $raw_user_info }}
{{- $raw_database_url := urlJoin $url_parts }}
{{- replace "$DATABASE_PASSWORD" "$(DATABASE_PASSWORD)" $raw_database_url }}
{{- end }}
