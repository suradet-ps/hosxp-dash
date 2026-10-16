import { invoke } from '@tauri-apps/api/core'

const A4_WIDTH_MM = 210
const A4_HEIGHT_MM = 297

/**
 * Renders a DOM element to an A4 PDF and saves it through the Rust side.
 *
 * The element is captured with html2canvas (so Thai text keeps the exact
 * shaping the UI shows) at 2x scale, then sliced across A4 pages and written
 * to disk by `save_pdf_file`. Returns the absolute path of the saved file.
 *
 * The PDF libraries are imported on demand so they stay out of the initial
 * bundle and load only on the first export.
 */
export async function exportElementToPdf(element: HTMLElement, fileName: string): Promise<string> {
  const [{ default: html2canvas }, { jsPDF }] = await Promise.all([
    import('html2canvas-pro'),
    import('jspdf'),
  ])

  const canvas = await html2canvas(element, {
    scale: 2,
    backgroundColor: '#ffffff',
    logging: false,
  })

  const pdf = new jsPDF({ unit: 'mm', format: 'a4', orientation: 'portrait' })
  const imageWidth = A4_WIDTH_MM
  const imageHeight = (canvas.height * imageWidth) / canvas.width

  // Very slight overflow is common with pixel rounding; squeeze it into a
  // single page instead of spilling a sliver onto page two.
  if (imageHeight <= A4_HEIGHT_MM * 1.06) {
    pdf.addImage(
      canvas.toDataURL('image/jpeg', 0.92),
      'JPEG',
      0,
      0,
      imageWidth,
      Math.min(imageHeight, A4_HEIGHT_MM),
    )
  } else {
    const pageHeightPx = Math.floor((canvas.width * A4_HEIGHT_MM) / A4_WIDTH_MM)
    let offsetY = 0
    let page = 0

    while (offsetY < canvas.height) {
      const sliceHeight = Math.min(pageHeightPx, canvas.height - offsetY)

      const slice = document.createElement('canvas')
      slice.width = canvas.width
      slice.height = sliceHeight
      const context = slice.getContext('2d')
      if (!context) throw new Error('ไม่สามารถเตรียมพื้นที่สำหรับไฟล์ PDF ได้')

      context.fillStyle = '#ffffff'
      context.fillRect(0, 0, slice.width, slice.height)
      context.drawImage(canvas, 0, offsetY, canvas.width, sliceHeight, 0, 0, canvas.width, sliceHeight)

      if (page > 0) pdf.addPage()
      const sliceHeightMm = (sliceHeight * imageWidth) / canvas.width
      pdf.addImage(slice.toDataURL('image/jpeg', 0.92), 'JPEG', 0, 0, imageWidth, sliceHeightMm)

      offsetY += sliceHeight
      page += 1
    }
  }

  const dataUri = pdf.output('datauristring')
  const base64 = dataUri.split(',')[1] ?? ''
  if (!base64) throw new Error('ไม่สามารถสร้างไฟล์ PDF ได้')

  return await invoke<string>('save_pdf_file', { fileName, base64Data: base64 })
}
