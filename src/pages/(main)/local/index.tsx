import { useState, useEffect, useMemo, useCallback } from 'react'
import { useAtomValue } from 'jotai'
import { useTranslation } from 'react-i18next'
import { Button } from '~/components/ui/button'
import { Input } from '~/components/ui/input'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '~/components/ui/table'
import { Checkbox } from '~/components/ui/checkbox'
import { audioService } from '~/services/audio-service'
import { scannerService } from '~/services/scanner-service'
import type { Song } from '~/types/bindings'
import { resolveImageUrl } from '~/lib/image'
import IconPlay from '~/assets/icons/icon_play.svg?react'
import IconPause from '~/assets/icons/icon_pause.svg?react'
import { musicIdAtom, musicPlayingAtom } from '~/atoms/player/index'

interface ScannerStatus {
  state: 'Idle' | 'Scanning' | 'Watching' | 'Stopped' | 'Not initialized'
  message?: string
}


function formatDuration(seconds?: number): string {
  if (!seconds) return '--:--'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`
}

function formatSize(bytes?: number): string {
  if (!bytes) return '--'
  const mb = bytes / (1024 * 1024)
  return `${mb.toFixed(1)}M`
}

function formatBitrate(bitrate?: number): string {
  if (!bitrate) return '--'
  return `${Math.round(bitrate / 1000)}kbps`
}

// removed unused getCoverImageUrl

export function Component() {
  const { t } = useTranslation('app')
  const [songs, setSongs] = useState<Song[]>([])
  const [loading, setLoading] = useState(true)
  const [selectedTracks, setSelectedTracks] = useState<Set<string>>(new Set())
  const [searchQuery, setSearchQuery] = useState('')
  const [sortBy] = useState<'title' | 'artist' | 'album' | 'duration'>('title')
  const [sortOrder] = useState<'asc' | 'desc'>('asc')
  const [scannerStatus, setScannerStatus] = useState<ScannerStatus>({ state: 'Not initialized' })
  // removed unused isPlaying

  const loadSongs = useCallback(async () => {
    try {
      setLoading(true)
      const localSongs = await scannerService.getLocalSongs()
      console.log('localSongs', localSongs)
      setSongs(localSongs)
      setLoading(false)
    } catch (error) {
      console.error('Failed to load local songs:', error)
      setLoading(false)
    }
  }, [])

  const checkScannerStatus = useCallback(async () => {
    try {
      const status = await scannerService.getStatus()
      setScannerStatus({ state: status as any })
    } catch (error) {
      console.error('Failed to get scanner status:', error)
      setScannerStatus({ state: 'Not initialized', message: t('pages.local.scanner.not_initialized') })
    }
  }, [])

  const startAutoScanner = useCallback(async () => {
    try {
      await scannerService.startAutoScanner()
      await checkScannerStatus()
    } catch (error) {
      console.error('Failed to start auto scanner:', error)
    }
  }, [checkScannerStatus])

  const triggerManualScan = useCallback(async () => {
    try {
      await scannerService.triggerManualScan()
      // setTimeout(loadSongs, 2000)
    } catch (error) {
      console.error('Manual scan failed:', error)
    }
  }, [loadSongs])

  useEffect(() => {
    loadSongs()
    checkScannerStatus()

    const unsubscribeStarted = scannerService.on('scanner-started', () => {
      setScannerStatus({ state: 'Watching' })
    })

    const unsubscribeStopped = scannerService.on('scanner-stopped', () => {
      setScannerStatus({ state: 'Stopped' })
    })

    const unsubscribeScanTriggered = scannerService.on('scan-triggered', () => {
      setScannerStatus({ state: 'Scanning' })
      setTimeout(() => {
        loadSongs()
        checkScannerStatus()
      }, 3000)
    })

    const unsubscribeError = scannerService.on('scanner-error', (error: any) => {
      console.error('Scanner error:', error)
      setScannerStatus({ state: 'Stopped', message: 'Scanner error' })
    })

    const unsubscribeScanProgress = scannerService.on('scan-progress', (result: any) => {
      console.log('Scan progress:', result)
    })

    const unsubscribeSongsAdded = scannerService.on('songs-added', (count: number) => {
      console.log(`Added ${count} songs`)
      loadSongs()
    })

    return () => {
      unsubscribeStarted()
      unsubscribeStopped() 
      unsubscribeScanTriggered()
      unsubscribeError()
      unsubscribeScanProgress()
      unsubscribeSongsAdded()
    }
  }, [loadSongs, checkScannerStatus])

  const filteredAndSortedSongs = useMemo(() => {
    let filtered = songs.filter(song => 
      song.title?.toLowerCase().includes(searchQuery.toLowerCase()) ||
      song.artists?.[0]?.artist_name?.toLowerCase().includes(searchQuery.toLowerCase()) ||
      song.album?.album_name?.toLowerCase().includes(searchQuery.toLowerCase())
    )

    filtered.sort((a, b) => {
      let aValue: string | number
      let bValue: string | number

      switch (sortBy) {
        case 'title':
          aValue = a.title || ''
          bValue = b.title || ''
          break
        case 'artist':
          aValue = a.artists?.[0]?.artist_name || ''
          bValue = b.artists?.[0]?.artist_name || ''
          break
        case 'album':
          aValue = a.album?.album_name || ''
          bValue = b.album?.album_name || ''
          break
        case 'duration':
          aValue = a.duration || 0
          bValue = b.duration || 0
          break
        default:
          return 0
      }

      if (typeof aValue === 'string' && typeof bValue === 'string') {
        return sortOrder === 'asc' 
          ? aValue.localeCompare(bValue)
          : bValue.localeCompare(aValue)
      } else {
        return sortOrder === 'asc' 
          ? (aValue as number) - (bValue as number)
          : (bValue as number) - (aValue as number)
      }
    })

    return filtered
  }, [songs, searchQuery, sortBy, sortOrder])

  const handleSelectTrack = useCallback((id: string) => {
    const newSelected = new Set(selectedTracks)
    if (newSelected.has(id)) {
      newSelected.delete(id)
    } else {
      newSelected.add(id)
    }
    setSelectedTracks(newSelected)
  }, [selectedTracks])

  const handleSelectAll = useCallback((checked: boolean) => {
    if (checked) {
      setSelectedTracks(new Set(filteredAndSortedSongs.map(s => s._id!).filter(Boolean)))
    } else {
      setSelectedTracks(new Set())
    }
  }, [filteredAndSortedSongs])

  const handlePlayTrack = useCallback(async (song: Song) => {
    try {
      await audioService.playSong(song)
    } catch (error) {
      console.error('Failed to play song:', error)
    }
  }, [filteredAndSortedSongs])

  // read global player state for button/icon rendering
  const currentPlayingId = useAtomValue(musicIdAtom)
  const isGlobalPlaying = useAtomValue(musicPlayingAtom)

  const handlePlayAll = useCallback(async () => {
    try {
      if (filteredAndSortedSongs.length > 0) {
        await audioService.playPlaylist(filteredAndSortedSongs, 0)
      }
    } catch (error) {
      console.error('Failed to play all:', error)
    }
  }, [filteredAndSortedSongs])

  const handleAddToQueue = useCallback(async () => {
    try {
      const selectedSongs = filteredAndSortedSongs.filter(s => selectedTracks.has(s._id!))
      if (selectedSongs.length > 0) {
        await audioService.addSongsToQueue(selectedSongs)
        setSelectedTracks(new Set())
      }
    } catch (error) {
      console.error('Failed to add to queue:', error)
    }
  }, [filteredAndSortedSongs, selectedTracks])

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="border-b border-border p-6">
        <div className="flex items-center justify-between mb-6">
          <h1 className="text-2xl font-bold text-text-primary">{t('pages.local.title')}</h1>
          
          <div className="flex items-center gap-3">
            <div className="flex items-center gap-2 text-sm text-text-secondary">
              <div className={`w-2 h-2 rounded-full ${
                scannerStatus.state === 'Watching' ? 'bg-green-500' :
                scannerStatus.state === 'Scanning' ? 'bg-yellow-500 animate-pulse' :
                scannerStatus.state === 'Stopped' ? 'bg-red-500' :
                'bg-gray-400'
              }`} />
              <span>{
                scannerStatus.state === 'Watching' ? t('pages.local.scanner.watching') :
                scannerStatus.state === 'Scanning' ? t('pages.local.scanner.scanning') :
                scannerStatus.state === 'Stopped' ? t('pages.local.scanner.stopped') :
                scannerStatus.state === 'Idle' ? t('pages.local.scanner.idle') :
                t('pages.local.scanner.not_initialized')
              }</span>
            </div>
            
            {scannerStatus.state === 'Not initialized' && (
              <Button 
                variant="ghost" 
                size="sm"
                onClick={startAutoScanner}
              >
                {t('pages.local.scanner.start')}
              </Button>
            )}
            
            <Button 
              variant="ghost" 
              size="sm"
              onClick={triggerManualScan}
              disabled={scannerStatus.state === 'Scanning'}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round"/>
              </svg>
              {scannerStatus.state === 'Scanning' ? t('pages.local.status.scanning') : t('pages.local.refresh')}
            </Button>
          </div>
        </div>

        {/* Controls */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <Button 
              variant="primary" 
              size="sm"
              onClick={handlePlayAll}
              disabled={filteredAndSortedSongs.length === 0}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M8 5v14l11-7z" fill="currentColor"/>
              </svg>
              {t('pages.local.play_all')}
            </Button>
            
            <Button 
              variant="ghost" 
              size="sm"
              onClick={handleAddToQueue}
              disabled={selectedTracks.size === 0}
            >
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none">
                <path d="M19 13h-6v6h-2v-6H5v-2h6V5h2v6h6v2z" fill="currentColor"/>
              </svg>
              {t('pages.local.add_to_queue')} ({selectedTracks.size})
            </Button>
          </div>

          <div className="flex items-center gap-3">
            <div className="relative">
              <Input
                placeholder={t('pages.local.search_placeholder')}
                value={searchQuery}
                onChange={(e) => setSearchQuery(e.target.value)}
                className="w-64 pl-9"
              />
              <svg 
                width="16" 
                height="16" 
                viewBox="0 0 24 24" 
                fill="none" 
                className="absolute left-3 top-1/2 -translate-y-1/2 text-text-tertiary"
              >
                <circle cx="11" cy="11" r="8" stroke="currentColor" strokeWidth="2"/>
                <path d="m21 21-4.35-4.35" stroke="currentColor" strokeWidth="2"/>
              </svg>
            </div>
          </div>
        </div>
      </div>

      {/* Track List */}
      <div className="flex-1 overflow-hidden">
        {loading ? (
          <div className="flex flex-col items-center justify-center h-64 text-text-secondary">
            <div className="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin mb-4" />
            <p className="text-lg font-medium">{t('pages.local.status.loading')}</p>
          </div>
        ) : filteredAndSortedSongs.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-text-secondary">
            <svg width="48" height="48" viewBox="0 0 24 24" fill="none" className="mb-4 opacity-50">
              <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" fill="currentColor"/>
            </svg>
            <p className="text-lg font-medium mb-2">
              {songs.length === 0 ? t('pages.local.empty.no_songs') : t('pages.local.empty.no_match')}
            </p>
            <p className="text-sm">
              {songs.length === 0 
                ? t('pages.local.empty.suggestion_setup')
                : t('pages.local.empty.suggestion_adjust')
              }
            </p>
          </div>
        ) : (
          <Table containerClassName="h-full overflow-auto">
            <TableHeader>
              <TableRow>
                <TableHead className="w-12">
                  <Checkbox
                    checked={selectedTracks.size === filteredAndSortedSongs.length && filteredAndSortedSongs.length > 0}
                    indeterminate={selectedTracks.size > 0 && selectedTracks.size < filteredAndSortedSongs.length}
                    onCheckedChange={handleSelectAll}
                    className="border border-border/40"
                  />
                </TableHead>
                <TableHead className="w-12"></TableHead>
                <TableHead>{t('pages.local.column.title')}</TableHead>
                <TableHead className="w-24 text-right">{t('pages.local.column.duration')}</TableHead>
                <TableHead className="w-20 text-right">{t('pages.local.column.size')}</TableHead>
                <TableHead className="w-20 text-center">{t('pages.local.column.format')}</TableHead>
                <TableHead className="w-12"></TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {filteredAndSortedSongs.map((song) => (
                <TableRow
                  key={song._id}
                  className="group hover:bg-accent/5"
                  data-state={selectedTracks.has(song._id!) ? 'selected' : undefined}
                >
                  <TableCell>
                    <Checkbox
                      checked={selectedTracks.has(song._id!)}
                      onCheckedChange={() => handleSelectTrack(song._id!)}
                      onClick={(e) => e.stopPropagation()}
                      className="border border-border/40"
                    />
                  </TableCell>
                  <TableCell>
                    <div className="relative w-10 h-10 bg-accent/10 rounded overflow-hidden flex items-center justify-center flex-shrink-0 group">
                      {(() => {
                        const coverUrl = resolveImageUrl(song.song_coverPath_high)
                        return coverUrl ? (
                          <img 
                            src={coverUrl} 
                            alt=""
                            className="w-full h-full object-cover rounded"
                            onError={(e) => {
                              const lowResUrl = resolveImageUrl(song.song_coverPath_low)
                              if (lowResUrl && e.currentTarget.src !== lowResUrl) {
                                e.currentTarget.src = lowResUrl
                              } else {
                                e.currentTarget.style.display = 'none'
                                const parent = e.currentTarget.parentElement
                                if (parent) {
                                  parent.innerHTML = `
                                    <svg width="16" height="16" viewBox="0 0 24 24" fill="none" class="text-accent/60">
                                      <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" fill="currentColor"/>
                                    </svg>
                                  `
                                }
                              }
                            }}
                          />
                        ) : (
                          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="text-accent/60">
                            <path d="M12 3v10.55c-.59-.34-1.27-.55-2-.55-2.21 0-4 1.79-4 4s1.79 4 4 4 4-1.79 4-4V7h4V3h-6z" fill="currentColor"/>
                          </svg>
                        )
                      })()}
                      <div className="absolute inset-0 rounded opacity-0 group-hover:opacity-100 transition-opacity duration-200 flex items-center justify-center
                        bg-[rgba(255,255,255,0.04)] dark:bg-[rgba(0,0,0,0.08)] backdrop-blur-[2px] border border-transparent shadow-[inset_0_1px_0_0_rgba(255,255,255,0.06)]">
                        {(() => {
                          const isCurrent = !!song._id && currentPlayingId === song._id
                          const showPause = isCurrent && isGlobalPlaying
                          const onClick = async (e: React.MouseEvent) => {
                            e.stopPropagation()
                            try {
                              if (isCurrent) {
                                if (isGlobalPlaying) {
                                  await audioService.pause()
                                } else {
                                  await audioService.play()
                                }
                              } else {
                                await handlePlayTrack(song)
                              }
                            } catch (err) {
                              console.error('change play state failed:', err)
                            }
                          }
                          return (
                            <button
                              onClick={onClick}
                              className="press-anim-parent text-white hover:text-accent transition-colors duration-200 flex items-center justify-center"
                            >
                              {showPause ? (
                                <IconPause className="w-4 h-4 press-anim" />
                              ) : (
                                <IconPlay className="w-4 h-4 press-anim" />
                              )}
                            </button>
                          )
                        })()}
                      </div>
                    </div>
                  </TableCell>
                  <TableCell>
                    <div className="min-w-0">
                      <div className="flex items-center gap-2">
                        <h3 className="text-sm font-medium text-text-primary truncate">
                          {song.title || t('pages.local.unknown.song')}
                        </h3>
                        {song.codec === 'FLAC' && (
                          <span className="px-1.5 py-0.5 text-xs bg-accent/20 text-accent rounded">{t('pages.local.lossless')}</span>
                        )}
                      </div>
                      <p className="text-xs text-text-tertiary truncate">
                        {song.artists?.[0]?.artist_name || t('pages.local.unknown.artist')} • {song.album?.album_name || t('pages.local.unknown.album')}
                      </p>
                    </div>
                  </TableCell>
                  <TableCell className="text-right text-text-secondary">
                    {formatDuration(song.duration ?? undefined)}
                  </TableCell>
                  <TableCell className="text-right text-text-secondary">
                    {formatSize(song.size ?? undefined)}
                  </TableCell>
                  <TableCell className="text-center">
                    <span className="text-xs text-text-tertiary">
                      {song.codec || '--'}
                    </span>
                    {song.bitrate && (
                      <div className="text-xs text-text-tertiary opacity-60">
                        {formatBitrate(song.bitrate)}
                      </div>
                    )}
                  </TableCell>
                  <TableCell>
                    <button
                      className="opacity-0 group-hover:opacity-100 p-1 hover:bg-accent/20 rounded transition-all"
                      onClick={(e) => {
                        e.stopPropagation()
                        // TODO: Show context menu
                      }}
                    >
                      <svg width="16" height="16" viewBox="0 0 24 24" fill="none" className="text-text-secondary">
                        <circle cx="12" cy="12" r="1" fill="currentColor"/>
                        <circle cx="19" cy="12" r="1" fill="currentColor"/>
                        <circle cx="5" cy="12" r="1" fill="currentColor"/>
                      </svg>
                    </button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </div>
      {filteredAndSortedSongs.length > 0 && (
        <div className="border-t border-border px-6 py-3 text-sm text-text-secondary bg-background-secondary">
          {t('pages.local.status.total')} {filteredAndSortedSongs.length} {t('pages.local.status.songs_count')}
          {songs.length !== filteredAndSortedSongs.length && ` (${t('pages.local.status.filtered')} ${songs.length - filteredAndSortedSongs.length} 首)`}
          {selectedTracks.size > 0 && ` • ${t('pages.local.status.selected')} ${selectedTracks.size} 首`}
        </div>
      )}
    </div>
  )
}
